use strum::EnumIter;
#[derive(Debug, EnumIter)]
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
