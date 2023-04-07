use serde::{Deserialize, Serialize};
use strum::EnumIter;
#[derive(Hash, Debug, EnumIter, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Dealer {
    Rema1000,
    Netto,
    DagliBrugsen,
    SuperBrugsen,
    Aldi,
    Bilka,
    Coop365,
    Irma,
    Føtex,
    Lidl,
    MENY,
    Kvickly,
    SPAR,
}

impl Dealer {
    pub fn id(&self) -> &'static str {
        match self {
            Dealer::Rema1000 => "11deC",
            Dealer::Netto => "9ba51",
            Dealer::DagliBrugsen => "d311fg",
            Dealer::Aldi => "98b7e",
            Dealer::Bilka => "93f13",
            Dealer::Coop365 => "DWZE1w",
            Dealer::Irma => "d432U",
            Dealer::Føtex => "bdf5A",
            Dealer::Lidl => "71c90",
            Dealer::MENY => "267e1m",
            Dealer::Kvickly => "c1edq",
            Dealer::SPAR => "88ddE",
            Dealer::SuperBrugsen => "0b1e8",
        }
    }
}

pub fn dealer_from_string(dealer_name: &str) -> Dealer {
    match dealer_name.to_lowercase().as_str() {
        "bilka" => Dealer::Bilka,
        "coop365" => Dealer::Coop365,
        "lidl" => Dealer::Lidl,
        "rema1000" => Dealer::Rema1000,
        "spar" => Dealer::SPAR,
        "meny" => Dealer::MENY,
        "føtex" => Dealer::Føtex,
        "irma" => Dealer::Irma,
        "aldi" => Dealer::Aldi,
        "netto" => Dealer::Netto,
        "kvickly" => Dealer::Kvickly,
        "daglibrugsen" | "dagli'brugsen" => Dealer::DagliBrugsen,
        "superbrugsen" => Dealer::SuperBrugsen,
        _ => panic!("Tried to look up dealer that doesn't exist"),
    }
}
