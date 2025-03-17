use anyhow::Context;
use chrono::{DateTime, NaiveDate, Utc};
use std::{collections::HashSet, str::FromStr};

use serde::{Deserialize, Serialize};

use super::dealer::Dealer;
use crate::Offer;
use comfy_table::{Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};
use futures::future;

#[derive(Serialize, Deserialize)]
pub(crate) struct UserData {
    favorites: HashSet<Dealer>,
    date_of_last_cache: NaiveDate,
    offers: Vec<Offer>,
    favorites_changed: bool,
}

impl UserData {
    pub fn print_favorites(&self) -> &HashSet<Dealer> {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec!["Favorites"]);

        for favorite in &self.favorites {
            table.add_row(vec![favorite]);
        }
        println!("{}", table);
        &self.favorites
    }

    pub(crate) fn from_cache() -> Option<UserData> {
        let path = dirs::cache_dir()?.join("etilbudsavis-cli/userdata.json");
        match std::fs::read_to_string(path) {
            Ok(data) => serde_json::from_str(&data).ok(),
            Err(_) => None,
        }
    }

    fn update_cache(&mut self) -> anyhow::Result<()> {
        self.date_of_last_cache = Utc::now().date_naive();
        let path = dirs::cache_dir()
            .context("Could not find cache dir")?
            .join("etilbudsavis-cli");
        std::fs::create_dir_all(&path)?;
        std::fs::write(path.join("userdata.json"), serde_json::to_string(&self)?)?;
        Ok(())
    }

    pub(crate) fn add_favorites(&mut self, dealers: &[Dealer]) {
        for &dealer in dealers {
            self.favorites_changed |= self.favorites.insert(dealer)
        }
    }

    pub(crate) fn remove_favorites(&mut self, dealers: &[Dealer]) {
        for dealer in dealers {
            self.favorites_changed |= self.favorites.remove(dealer)
        }
    }

    #[inline(always)]
    fn cache_outdated(&self) -> bool {
        self.date_of_last_cache < Utc::now().date_naive()
    }

    /// Fetch offers if we have updates or the cache is too old.
    pub(crate) async fn retrieve_offers(&mut self) {
        if self.favorites_changed || self.cache_outdated() {
            self.offers = self.retrieve_offers_from_remote().await;
            if let Err(err) = self.update_cache() {
                eprintln!("Failed to update cache: {}", err);
            }
        }
    }

    async fn retrieve_offers_from_remote(&mut self) -> Vec<Offer> {
        let tasks: Vec<_> = self
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

    pub async fn search(&mut self, search_items: &[String], search_by_dealer: bool) -> Vec<&Offer> {
        if search_items.is_empty() {
            self.retrieve_offers().await;
            return self.offers.iter().collect();
        }

        search_items
            .iter()
            .flat_map(|search| {
                if search_by_dealer {
                    if let Ok(dealer) = Dealer::from_str(search) {
                        self.offers
                            .iter()
                            .filter(|offer| offer.dealer == dealer)
                            .collect::<Vec<_>>()
                    } else {
                        println!("Search term did not match any known dealers: {search}");
                        Dealer::list_known_dealers(None);
                        vec![]
                    }
                } else {
                    self.offers
                        .iter()
                        .filter(|offer| offer.name.to_lowercase().contains(search.trim()))
                        .collect::<Vec<_>>()
                }
            })
            .collect()
    }
}

impl Default for UserData {
    fn default() -> Self {
        UserData {
            date_of_last_cache: DateTime::UNIX_EPOCH.date_naive(),
            favorites_changed: false,
            favorites: HashSet::new(),
            offers: Vec::new(),
        }
    }
}
