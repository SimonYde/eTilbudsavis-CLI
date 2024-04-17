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
        Err(err) => {
            eprintln!("{}", err);
            UserData::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct UserData {
    pub(crate) favorites: HashSet<Dealer>,
    date_of_last_cache: NaiveDate,
}

impl UserData {
    pub(crate) fn save(&self) {
        let path = dirs::cache_dir().unwrap().join("better_tilbudsavis");
        std::fs::create_dir_all(path.clone()).unwrap();
        std::fs::write(
            path.join("userdata.json"),
            serde_json::to_string(&self).unwrap(),
        )
        .unwrap();
    }
    pub(crate) fn should_update_cache(&self) -> bool {
        self.date_of_last_cache < Utc::now().date_naive()
    }
    pub(crate) fn cache_updated(&mut self) {
        self.date_of_last_cache = Utc::now().date_naive();
        self.save();
    }
}

impl Default for UserData {
    fn default() -> Self {
        println!("First run, or userdata was modified outside of running of this program.");
        println!("Reinitialising userdata with no favorites...");
        UserData {
            favorites: HashSet::new(),
            date_of_last_cache: Utc.timestamp_millis_opt(0).unwrap().date_naive(),
        }
    }
}
