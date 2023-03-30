mod dealer;
use crate::dealer::Dealer;
mod deserialize;
use crate::deserialize::*;

use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, Response,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() {
    let mut offers: Vec<Offer> = vec![];
    for dealer in Dealer::iter() {
        offers.append(&mut retrieve_offers_from_dealer(&dealer).await.unwrap());
    }
    offers.retain(|offer| offer.name.contains("Pepsi"));
    println!("{:?}", offers);
}
// let offer = &offer_wrapper["offer"];
// let quantity = &offer["quantity"];
// let amount = &quantity["pieces"];
// let size = &quantity["size"];
// let unit = &quantity["unit"]["si"];
// let factor = &unit["factor"].as_f64()?;
// min_amount: amount["from"].as_u64()? as u32,
// max_amount: amount["to"].as_u64()? as u32,
// min_size: size["from"].as_f64()? * factor,
// max_size: size["to"].as_f64()? * factor,
// unit: unit["symbol"].as_str()?.to_owned(),
// run_from: offer["run_from"].as_str()?.split('T').next()?.to_string(),
// run_till: offer["run_till"].as_str()?.split('T').next()?.to_string(),
// dealer: dealer_name.to_string(),

#[derive(Serialize, Deserialize, Debug)]
struct Offer {
    // id: String,
    #[serde(rename = "heading")]
    name: String,
    #[serde(rename = "pricing", deserialize_with = "deserialize_offer_price")]
    price: f64,
    // min_amount: u32,
    // max_amount: u32,
    // min_size: f64,
    // max_size: f64,
    // unit: String,
    // run_from: String,
    // run_till: String,
    // dealer: String,
}

// impl Offer {
//     fn cost_per_unit(&self) -> f64 {
//         match self.unit.as_str() {
//             "kg" | "l" => self.price / self.max_size,
//             _ => self.price,
//         }
//     }
// }

#[derive(Serialize, Deserialize, Debug)]
struct Catalog {
    id: String,
    run_from: String,
    run_till: String,
    #[serde(rename = "dealer", deserialize_with = "deserialize_dealer_name")]
    dealer_name: String,
    offer_count: u32,
}

async fn retrieve_offers_from_dealer(dealer: &Dealer) -> Option<Vec<Offer>> {
    let client = Client::new();
    let catalog_response = request_catalogs(dealer, &client).await?;

    let catalogs = match catalog_response.status() {
        reqwest::StatusCode::OK => catalog_response.json::<Vec<Catalog>>().await.ok()?,
        _ => vec![],
    };

    let futures_offers = catalogs
        .into_iter()
        .map(|catalog| retrieve_offers_from_catalog(catalog, &client));
    let offers = futures::future::join_all(futures_offers)
        .await
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    Some(offers)
}

#[derive(Deserialize)]
struct OfferWrapper {
    offer: Offer,
}

async fn retrieve_offers_from_catalog(catalog: Catalog, client: &Client) -> Option<Vec<Offer>> {
    let offers_response = client
        .get(format!(
            "https://squid-api.tjek.com/v2/catalogs/{}/hotspots",
            catalog.id.as_str()
        ))
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .send()
        .await
        .ok()?;
    let offers = offers_response
        .json::<Vec<OfferWrapper>>()
        .await
        .ok()?
        .into_iter()
        .map(|ow| ow.offer)
        .collect();
    Some(offers)
}

async fn request_catalogs(dealer: &Dealer, client: &Client) -> Option<Response> {
    client
        .get("https://squid-api.tjek.com/v2/catalogs")
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .query(&[("dealer_ids", dealer.id())])
        .send()
        .await
        .ok()
}
