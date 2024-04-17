use chrono::{NaiveDate, TimeZone, Utc};
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::dealer::Dealer;

pub(crate) fn get_userdata() -> UserData {
    match std::fs::read_to_string("./data/userdata.json") {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => UserData::default(),
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct UserData {
    pub(crate) favorites: HashSet<Dealer>,
    pub(crate) time_of_last_cache: NaiveDate,
}

impl UserData {
    pub(crate) fn save(&self) {
        std::fs::write("./data/userdata.json", serde_json::to_string(&self).unwrap()).unwrap();
    }
    pub(crate) fn should_update_cache(&self) -> bool {
        self.time_of_last_cache < Utc::now().date_naive()
    }

}

impl Default for UserData {
    fn default() -> Self {
        println!("First run, or userdata was modified outside of running of this program.");
        println!("Reinitialising userdata with no favorites...");
        UserData {
            favorites: HashSet::new(),
            time_of_last_cache: Utc.timestamp_millis_opt(0).unwrap().date_naive(),
        }
    }
}
