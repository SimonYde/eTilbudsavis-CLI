use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::userdata::UserData;

#[derive(Debug, Deserialize, Serialize, PartialOrd)]
pub(crate) struct Offer {
    // id: String,
    pub(crate) name: String,
    pub(crate) dealer: String,
    pub(crate) price: f64,
    pub(crate) cost_per_unit: f64,
    pub(crate) unit: String,
    pub(crate) min_size: f64,
    pub(crate) max_size: f64,
    pub(crate) min_amount: u32,
    pub(crate) max_amount: u32,
    pub(crate) run_from: NaiveDate,
    pub(crate) run_till: NaiveDate,
}

impl PartialEq for Offer {
    fn eq(&self, other: &Self) -> bool {
        self.dealer == other.dealer
            && self.name == other.name
            && self.run_from == other.run_from
            && self.run_till == other.run_till
    }
}

impl Eq for Offer {}

impl Default for Offer {
    fn default() -> Self {
        Offer {
            name: String::default(),
            dealer: String::default(),
            price: f64::default(),
            cost_per_unit: f64::default(),
            unit: String::default(),
            min_size: f64::default(),
            max_size: f64::default(),
            min_amount: u32::default(),
            max_amount: u32::default(),
            run_from: Utc::now().date_naive(),
            run_till: Utc::now().date_naive(),
        }
    }
}

pub(crate) fn cache_retrieved_offers(userdata: &mut UserData, offers: &Vec<Offer>) -> Result<()> {
    let path = dirs::cache_dir().unwrap().join("better_tilbudsavis");
    std::fs::create_dir_all(path.clone())?;
    std::fs::write(
        path.join("offer_cache.json"),
        serde_json::to_string(offers).context("Failed to serialize offers to JSON")?,
    )
    .context("could not write offer cache")?;
    println!("WRITTEN offer cache");
    userdata.cache_updated();
    Ok(())
}

pub(crate) fn retrieve_cached_offers() -> Result<Vec<Offer>> {
    let path = dirs::cache_dir()
        .unwrap()
        .join("better_tilbudsavis/offer_cache.json");
    let offer_cache_str = &match std::fs::read_to_string(path) {
        Ok(cache) => cache,
        Err(err) => err.to_string(),
    };
    serde_json::from_str(offer_cache_str).context("Offer cache has invalid JSON")
}

pub(crate) async fn retrieve_offers(
    userdata: &mut UserData,
    favorites_changed: bool,
) -> Vec<Offer> {
    match retrieve_cached_offers() {
        Ok(offers) => {
            let cache_outdated = userdata.should_update_cache();
            if favorites_changed || cache_outdated {
                let offers = retrieve_offers_from_remote(userdata).await;
                cache_retrieved_offers(userdata, &offers)
                    .expect("Should be able to write cache to file");
                return offers;
            }
            offers
        }
        Err(_) => {
            let offers = retrieve_offers_from_remote(userdata).await;
            cache_retrieved_offers(userdata, &offers).unwrap();
            offers
        }
    }
}

pub(crate) async fn retrieve_offers_from_remote(userdata: &mut UserData) -> Vec<Offer> {
    futures::future::join_all(
        userdata
            .favorites
            .iter()
            .map(|dealer| dealer.remote_offers_for_dealer()),
    )
    .await
    .into_iter()
    .flatten()
    .collect()
}
