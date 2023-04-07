mod dealer;
use crate::dealer::*;
mod deserialize;
use crate::deserialize::*;

use chrono::{NaiveDate, Utc};
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{env, fs};
use strum::IntoEnumIterator;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let userdata = match fs::read_to_string("./userdata.json") {
        Ok(data) => serde_json::from_str(&data).unwrap(),
        Err(_) => UserData {
            favorites: HashSet::new(),
            rerun_date: Utc::now()
                .date_naive()
                .checked_sub_days(chrono::Days::new(1))
                .unwrap()
                .to_string(),
        }, // Never run before
    };
    let mut offers = parse_args(userdata, args).await;

    println!("Amount of offers: {}", offers.len());
    offers.sort_by(|a, b| a.cost_per_unit.total_cmp(&b.cost_per_unit).reverse());
    offers.iter().for_each(|offer| println!("{:?}", offer));
}

async fn parse_args(mut userdata: UserData, args: Vec<String>) -> Vec<Offer> {
    let mut favorites_changed = false;
    let mut offers = vec![];
    if args.len() > 1 {
        match args[1].as_str() {
            "--search" | "-s" => {
                offers = retrieve_offers(userdata, favorites_changed).await;
                println!("Amount of offers: {}", offers.len());
                if args.len() > 2 && !args[2].starts_with("--") {
                    println!("searching for: {}", args[2]);
                    offers.retain(|offer| offer.name.to_lowercase().contains(&args[2]));
                } else {
                    println!("No search term provided, not filtering offers...")
                }
                offers
            }
            "--favorites" | "-f" => {
                println!("test favourites");
                if args.len() > 2 {
                    match args[2].to_lowercase().as_str() {
                        "bilka" | "coop365" | "lidl" | "rema1000" | "spar" | "meny" | "føtex"
                        | "irma" | "aldi" | "netto" | "kvickly" | "daglibrugsen"
                        | "dagli'brugsen" | "superbrugsen" => {
                            userdata.favorites.insert(dealer_from_string(&args[2]));
                        }
                        _ => panic!("Argument to \"--favorites\" was invalid."),
                    }
                    favorites_changed = true;

                    fs::write("./userdata.json", serde_json::to_string(&userdata).unwrap())
                        .expect("failed to write UserData");
                    retrieve_offers(userdata, favorites_changed).await
                } else {
                    offers
                }
            }
            _ => panic!("invalid arguments given"),
        }
    } else {
        println!("args empty");
        offers
    }
}

async fn retrieve_offers(mut userdata: UserData, favorites_changed: bool) -> Vec<Offer> {
    let mut offers = vec![];
    let today = Utc::now().date_naive();
    let naive_date =
        NaiveDate::parse_from_str(&userdata.rerun_date, "%Y-%m-%d").expect("Date has wrong format");
    let diff = today.signed_duration_since(naive_date);
    if diff.num_days() > 0 || favorites_changed {
        for dealer in &userdata.favorites {
            offers.append(
                &mut retrieve_offers_from_remote(dealer)
                    .await
                    .expect("Failed to retrieve remote offers"),
            );
        }
        cache_retrieved_offers(&offers, &mut userdata).expect("Was unable to cache offers");
    } else {
        offers = retrieve_cached_offers().expect("Was unable to receive offers from cache");
    }
    offers
}

#[derive(Serialize, Deserialize)]
struct UserData {
    favorites: HashSet<Dealer>,
    rerun_date: String,
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

#[derive(Deserialize)]
struct Catalog {
    id: String,
    #[serde(deserialize_with = "deserialize_dealer_name")]
    dealer: String,
    // offer_count: u32,
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

fn cache_retrieved_offers(offers: &Vec<Offer>, userdata: &mut UserData) -> std::io::Result<()> {
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
    userdata.rerun_date = most_soon_to_run_out.to_string();
    fs::write("./userdata.json", serde_json::to_string(userdata).unwrap())?;
    println!("WRITTEN date");
    Ok(())
}

fn retrieve_cached_offers() -> Option<Vec<Offer>> {
    // TODO Why can't I retrieve from subdir? such as ./data/cached_offers.json
    serde_json::from_str(
        &fs::read_to_string("./cached_offers.json").expect("Cannot open file: cached_offers.json"),
    )
    .ok()
}

async fn retrieve_catalogs_from_dealer(dealer: &Dealer, client: &Client) -> Option<Vec<Catalog>> {
    let catalog_response = request_catalogs(dealer, client).await?;
    match catalog_response.status() {
        reqwest::StatusCode::OK => catalog_response.json::<Vec<Catalog>>().await.ok(),
        _ => {
            println!("Did not succesfully access API, no connection?");
            Some(vec![])
        }
    }
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
