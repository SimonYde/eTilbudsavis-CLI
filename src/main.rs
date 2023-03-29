mod dealer;
use crate::dealer::Dealer;

use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, Response,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[tokio::main]
async fn main() {
    let mut offers_from_rema = retrieve_offers_from_dealer(&Dealer::Rema1000)
        .await
        .unwrap();
    offers_from_rema.truncate(6);
    println!("{:?}", offers_from_rema);
    offers_from_rema
        .iter()
        .map(Offer::cost_per_unit)
        .for_each(|f| print!("{:.1$}, ", f, 2));
    println!();
    let mut offers_from_netto = retrieve_offers_from_dealer(&Dealer::Netto).await.unwrap();
    offers_from_netto.truncate(6);
    println!("{:?}", offers_from_netto);
}

#[derive(Serialize, Deserialize, Debug)]
struct Offer {
    id: String,
    name: String,
    price: f64,
    min_amount: u32,
    max_amount: u32,
    min_size: f64,
    max_size: f64,
    unit: String,
    start_date: String,
    end_date: String,
}

impl Offer {
    fn cost_per_unit(&self) -> f64 {
        match self.unit.as_str() {
            "kg" | "l" => self.price / self.max_size,
            _ => self.price,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Catalog {
    id: String,
    run_from: String,
    run_till: String,
    dealer_id: String,
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
    let parsed = offers_response
        .json::<Vec<Value>>()
        .await
        .ok()?
        .into_iter()
        .filter_map(create_offer)
        .collect();
    Some(parsed)
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

fn create_offer(offer_wrapper: Value) -> Option<Offer> {
    let offer = &offer_wrapper["offer"];
    let quantity = &offer["quantity"];
    let amount = &quantity["pieces"];
    let size = &quantity["size"];
    let unit = &quantity["unit"]["si"];
    let factor = &unit["factor"].as_f64()?;
    Some(Offer {
        id: offer["id"].as_str()?.to_owned(),
        name: offer["heading"].as_str()?.to_owned(),
        price: offer["pricing"]["price"].as_f64()?,
        min_amount: amount["from"].as_u64()? as u32,
        max_amount: amount["to"].as_u64()? as u32,
        min_size: size["from"].as_f64()? * factor,
        max_size: size["to"].as_f64()? * factor,
        unit: unit["symbol"].as_str()?.to_owned(),
        start_date: offer["run_from"].as_str()?.split('T').next()?.to_string(),
        end_date: offer["run_till"].as_str()?.split('T').next()?.to_string(),
    })
}
