use anyhow::{anyhow, Context, Result};
use futures::future;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator};

use super::{
    deserialize::{deserialize_dealer_name, deserialize_offer, OfferWrapper},
    offer::Offer,
};
#[derive(
    Hash,
    Debug,
    EnumIter,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    Default,
    // ValueEnum,
)]
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
    Meny,
    Kvickly,
    #[default]
    Spar,
}

impl Dealer {
    fn id(&self) -> &'static str {
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
            Dealer::Meny => "267e1m",
            Dealer::Kvickly => "c1edq",
            Dealer::Spar => "88ddE",
            Dealer::SuperBrugsen => "0b1e8",
        }
    }
    pub(crate) fn list_known_dealers() {
        let mut table = comfy_table::Table::new();

        table
            .load_preset(comfy_table::presets::UTF8_FULL)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
            .set_header(vec!["Dealers"]);
        for dealer in Dealer::iter() {
            table.add_row(vec![dealer.to_string()]);
        }
        println!("{table}");
    }
    pub(crate) async fn remote_offers_for_dealer(&self) -> Vec<Offer> {
        let client = Client::new();
        let catalogs = retrieve_catalogs_from_dealer(self, &client)
            .await
            .unwrap_or_default();
        let tasks: Vec<_> = catalogs
            .into_iter()
            .map(|catalog| {
                let catalog = catalog.clone();
                let client = client.clone();
                tokio::spawn(async move { retrieve_offers_from_catalog(catalog, &client).await })
            })
            .collect();

        future::join_all(tasks)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .flatten()
            .collect()
    }
}

impl std::fmt::Display for Dealer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl FromStr for Dealer {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = match s.to_lowercase().trim() {
            "bilka" => Dealer::Bilka,
            "coop365" => Dealer::Coop365,
            "lidl" => Dealer::Lidl,
            "rema1000" | "rema 1000" => Dealer::Rema1000,
            "spar" => Dealer::Spar,
            "meny" => Dealer::Meny,
            "føtex" => Dealer::Føtex,
            "irma" => Dealer::Irma,
            "aldi" => Dealer::Aldi,
            "netto" => Dealer::Netto,
            "kvickly" => Dealer::Kvickly,
            "daglibrugsen" | "dagli'brugsen" => Dealer::DagliBrugsen,
            "superbrugsen" => Dealer::SuperBrugsen,
            _ => {
                return Err(anyhow!(
                    "Unknown dealer: {s}.\nSee `dealers` for available dealers."
                ))
            }
        };
        Ok(value)
    }
}

#[derive(Deserialize, Clone)]
struct Catalog {
    id: String,
    #[serde(deserialize_with = "deserialize_dealer_name")]
    dealer: String,
}

async fn retrieve_catalogs_from_dealer(dealer: &Dealer, client: &Client) -> Result<Vec<Catalog>> {
    let catalog_response = client
        .get("https://squid-api.tjek.com/v2/catalogs")
        .query(&[("dealer_ids", dealer.id())])
        .header("Accept", "application/json")
        .send()
        .await?;
    match catalog_response.status() {
        reqwest::StatusCode::OK => catalog_response
            .json::<Vec<Catalog>>()
            .await
            .context("Dealer returned invalid JSON"),
        _ => {
            println!(
                "Did not succesfully access API, StatusCode: {}",
                catalog_response.status()
            );
            Ok(Vec::new())
        }
    }
}

async fn retrieve_offers_from_catalog(catalog: Catalog, client: &Client) -> Result<Vec<Offer>> {
    let offers = client
        .get(format!(
            "https://squid-api.tjek.com/v2/catalogs/{}/hotspots",
            catalog.id.as_str()
        ))
        .send()
        .await
        .context("could not fetch offers")?
        .json::<Vec<OfferWrapper>>()
        .await
        .context("could not deserialize offers")?
        .into_iter()
        .map(|ow| deserialize_offer(ow, &catalog.dealer))
        .collect();
    Ok(offers)
}
