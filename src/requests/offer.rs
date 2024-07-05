use anyhow::{Context, Result};
use chrono::{NaiveDate, Utc};
use comfy_table::{Cell, CellAlignment};
use futures::future;
use serde::{Deserialize, Serialize};

use super::{dealer::Dealer, userdata::UserData};

#[derive(Debug, Deserialize, Serialize, PartialOrd)]
pub(crate) struct Offer {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) dealer: Dealer,
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
        self.id == other.id
            || (self.dealer == other.dealer
                && self.name == other.name
                && self.run_from == other.run_from
                && self.run_till == other.run_till)
    }
}

impl Eq for Offer {}

impl Default for Offer {
    fn default() -> Self {
        Offer {
            id: String::default(),
            name: String::default(),
            dealer: Dealer::default(),
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

impl std::fmt::Display for Offer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let offer_str = format!(
            "{} - {}: {} - {}: {} kr. - {:.2} kr/{}",
            self.run_from.format("%d/%m"),
            self.run_till.format("%d/%m"),
            self.dealer,
            self.name,
            self.price,
            self.cost_per_unit,
            self.unit
        );
        write!(f, "{}", offer_str)?;
        Ok(())
    }
}

impl Offer {
    pub(crate) fn to_table_entry(&self) -> Vec<Cell> {
        let unit = &self.unit;
        let period = format!(
            "{}\n  ↓  \n{}",
            self.run_from.format("%d/%m"),
            self.run_till.format("%d/%m")
        );
        let cost_per_unit = format!("{:.2} kr/{}", self.cost_per_unit, unit);
        let price = format!("{:.2} kr", self.price);
        let count = if self.min_amount == self.max_amount {
            format!("{}", self.min_amount)
        } else {
            format!("{}-{}", self.min_amount, self.max_amount)
        };

        let min_size_is_decimal = self.min_size - self.min_size.trunc() > 0.01;
        let max_size_is_decimal = self.max_size - self.max_size.trunc() > 0.01;
        let max_size_equals_min_size = self.max_size - self.min_size < 0.001;

        let min_size = if min_size_is_decimal {
            format!("{:.3}", self.min_size)
        } else {
            format!("{}", self.min_size)
        };
        let max_size = if max_size_is_decimal {
            format!("{:.3}", self.max_size)
        } else {
            format!("{}", self.max_size)
        };

        let weight = if max_size_equals_min_size {
            format!("{} {}", min_size, unit)
        } else {
            format!("{}-{} {}", min_size, max_size, unit)
        };

        vec![
            Cell::new(period),
            Cell::new(self.dealer.to_string()),
            Cell::new(self.name.to_string()),
            Cell::new(count),
            Cell::new(price).set_alignment(CellAlignment::Right),
            Cell::new(cost_per_unit).set_alignment(CellAlignment::Right),
            Cell::new(weight).set_alignment(CellAlignment::Right),
        ]
    }
}

pub(crate) async fn retrieve_offers(
    userdata: &mut UserData,
    favorites_changed: bool,
) -> Vec<Offer> {
    match retrieve_cached_offers() {
        Ok(cached_offers) => {
            let cache_outdated = userdata.should_update_cache();
            if favorites_changed || cache_outdated {
                let offers = retrieve_offers_from_remote(userdata).await;
                if let Err(err) = cache_retrieved_offers(userdata, &offers) {
                    eprintln!("{err}");
                }
                offers
            } else {
                cached_offers
            }
        }
        Err(_) => {
            let offers = retrieve_offers_from_remote(userdata).await;
            if let Err(err) = cache_retrieved_offers(userdata, &offers) {
                eprintln!("{err}");
            }
            offers
        }
    }
}

fn cache_retrieved_offers(userdata: &mut UserData, offers: &Vec<Offer>) -> Result<()> {
    let path = dirs::cache_dir()
        .context("Could not find cache dir")?
        .join("etilbudsavis-cli");
    std::fs::create_dir_all(path.clone())?;
    std::fs::write(
        path.join("offer_cache.json"),
        serde_json::to_string(offers).context("Failed to serialize offers to JSON")?,
    )
    .context("could not write offer cache")?;
    userdata.cache_updated();
    Ok(())
}

fn retrieve_cached_offers() -> Result<Vec<Offer>> {
    let path = dirs::cache_dir()
        .context("Could not find cache dir")?
        .join("etilbudsavis-cli/offer_cache.json");
    let offer_cache_str = std::fs::read_to_string(path).context("Offer cache not found")?;
    serde_json::from_str(&offer_cache_str).context("Offer cache has invalid JSON")
}

async fn retrieve_offers_from_remote(userdata: &mut UserData) -> Vec<Offer> {
    let tasks: Vec<_> = userdata
        .favorites
        .iter()
        .map(|dealer| {
            let dealer = *dealer;
            tokio::spawn(async move { dealer.remote_offers_for_dealer().await })
        })
        .collect();

    future::join_all(tasks)
        .await
        .into_iter()
        .flatten()
        .flatten()
        .collect()
}
