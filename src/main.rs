mod dealer;
use crate::dealer::Dealer;
mod deserialize;
use crate::deserialize::*;
mod test;
use chrono::{NaiveDate, Utc};
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, Response,
};
use serde::{Deserialize, Serialize};
use std::fs;
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() {
    let mut offers = retrieve_offers().await;
    println!("Amount of offers: {}", offers.len());
    offers.retain(|offer| offer.name.to_lowercase().contains("tuborg"));
    offers.sort_by(|a, b| a.cost_per_unit.total_cmp(&b.cost_per_unit));
    offers.into_iter().for_each(|offer| println!("{:?}", offer));
}

async fn retrieve_offers() -> Vec<Offer> {
    let mut offers = vec![];
    let today = Utc::now().date_naive();
    println!("Current date: {}", today);
    let date_string = match fs::read_to_string("./run.json") {
        Ok(date) => date,
        Err(_) => today
            .checked_sub_days(chrono::Days::new(5))
            .unwrap()
            .to_string(),
    };
    let date_of_catalog = NaiveDate::parse_from_str(&date_string, "%Y-%m-%d").unwrap();
    let diff = today.signed_duration_since(date_of_catalog);
    println!("{}", diff.num_days());
    if diff.num_days() > 0 {
        for dealer in Dealer::iter() {
            offers.append(&mut retrieve_offers_from_remote(&dealer).await.unwrap())
        }
        cache_retrieved_offers(&offers).expect("Was unable to cache offers");
    } else {
        offers = retrieve_cached_offers().expect("Was unable to receive offers from cache");
    }
    offers
}

#[derive(Deserialize)]
struct OfferWrapper {
    offer: Outer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Offer {
    // id: String,
    name: String,
    dealer: String,
    price: f64,
    cost_per_unit: f64,
    min_amount: u32,
    max_amount: u32,
    min_size: f64,
    max_size: f64,
    unit: String,
    run_from: String,
    run_till: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Catalog {
    id: String,
    run_from: String,
    run_till: String,
    #[serde(deserialize_with = "deserialize_dealer_name")]
    dealer: String,
    offer_count: u32,
}

async fn retrieve_offers_from_remote(dealer: &Dealer) -> Option<Vec<Offer>> {
    let client = Client::new();
    let catalogs = retrieve_catalogs_from_dealer(dealer, &client).await?;

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

fn cache_retrieved_offers(offers: &Vec<Offer>) -> std::io::Result<()> {
    fs::write(
        "./cached_offers.json",
        serde_json::to_string(offers).expect("Could not write \"cached offers\""),
    )?;
    println!("WRITTEN offer cache");
    let most_soon_to_run_out = offers
        .iter()
        .map(|o| &o.run_till)
        .map(|date| {
            NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .expect("Couldn't parse NaiveDate, did API format change?")
        })
        .min()
        .unwrap();
    println!("{}", most_soon_to_run_out);
    fs::write("./data/run.json", most_soon_to_run_out.to_string())?;
    println!("WRITTEN date");
    Ok(())
}

fn retrieve_cached_offers() -> Option<Vec<Offer>> {
    serde_json::from_str(
        &fs::read_to_string("./cached_offers.json").expect("Cannot open file: cached_offers.json"),
    )
    .ok()
}

async fn retrieve_catalogs_from_dealer(dealer: &Dealer, client: &Client) -> Option<Vec<Catalog>> {
    let catalog_response = request_catalogs(dealer, client).await?;
    let catalogs = match catalog_response.status() {
        reqwest::StatusCode::OK => catalog_response.json::<Vec<Catalog>>().await.ok()?,
        _ => {
            println!("Did not succesfully access API, no connection?");
            vec![]
        }
    };
    Some(catalogs)
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
        .map(|o| deserialize_offer(o, &catalog.dealer))
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
