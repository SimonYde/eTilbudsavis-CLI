use chrono::{NaiveDate, TimeZone, Utc};
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::dealer::Dealer;

pub(crate) fn get_userdata() -> UserData {
    let path = dirs::cache_dir()
        .unwrap()
        .join("better_tilbudsavis/userdata.json");
    match std::fs::read_to_string(path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => UserData::default(),
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct UserData {
    pub(crate) favorites: HashSet<Dealer>,
    date_of_last_cache: NaiveDate,
}

impl UserData {
    pub(crate) fn save(&self) -> anyhow::Result<()> {
        let path = dirs::cache_dir()
            .ok_or(anyhow::anyhow!("Failed to get cache dir"))?
            .join("better_tilbudsavis");
        std::fs::create_dir_all(path.clone())?;
        std::fs::write(path.join("userdata.json"), serde_json::to_string(&self)?)?;
        Ok(())
    }

    pub(crate) fn should_update_cache(&self) -> bool {
        self.date_of_last_cache < Utc::now().date_naive()
    }

    pub(crate) fn cache_updated(&mut self) {
        self.date_of_last_cache = Utc::now().date_naive();
        if let Err(err) = self.save() {
            eprintln!("Failed to save userdata: {}", err);
        };
    }

    pub(crate) fn add_favorites(&mut self, dealers: &[Dealer]) -> bool {
        let mut changed = false;
        for dealer in dealers {
            changed |= self.favorites.insert(*dealer)
        }
        changed
    }

    pub(crate) fn remove_favorites(&mut self, dealers: &[Dealer]) -> bool {
        let mut changed = false;
        for dealer in dealers {
            changed |= self.favorites.remove(dealer)
        }
        changed
    }
}

impl Default for UserData {
    fn default() -> Self {
        println!("Initializing userdata...");
        UserData {
            favorites: HashSet::new(),
            date_of_last_cache: Utc.timestamp_millis_opt(0).unwrap().date_naive(),
        }
    }
}
