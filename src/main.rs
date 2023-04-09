use chrono::{Days, NaiveDate, Utc};
use clap::Parser;
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
// use strum::IntoEnumIterator;

mod dealer;
use crate::dealer::*;
mod deserialize;
use crate::deserialize::*;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut userdata = match fs::read_to_string("data/userdata.json") {
        Ok(data) => serde_json::from_str(&data).expect("shouldn't happen"),
        Err(_) => UserData {
            favorites: HashSet::new(),
            rerun_date: Utc::now()
                .date_naive()
                .checked_sub_days(Days::new(1))
                .unwrap(),
        }, // Never run before
    };
    let favorites_changed =
        handle_favorites(&mut userdata, &args.add_favorites, &args.remove_favorites);
    let mut offers = handle_search(userdata, args.search, favorites_changed).await;
    offers.sort_by(|a, b| a.cost_per_unit.total_cmp(&b.cost_per_unit).reverse());

    if args.print {
        offers.iter().for_each(|offer| println!("{:?}", offer))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(use_value_delimiter = true, value_delimiter = ',', short, long)]
    add_favorites: Vec<String>,
    #[arg(use_value_delimiter = true, value_delimiter = ',', short, long)]
    remove_favorites: Vec<String>,
    #[arg(use_value_delimiter = true, value_delimiter = ',', short, long)]
    search: Vec<String>,

    #[arg(short, long)]
    print: bool,
}
#[derive(Serialize, Deserialize)]
struct UserData {
    favorites: HashSet<Dealer>,
    rerun_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
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
    run_from: NaiveDate,
    run_till: NaiveDate,
}

#[derive(Deserialize)]
struct Catalog {
    id: String,
    #[serde(deserialize_with = "deserialize_dealer_name")]
    dealer: String,
    // offer_count: u32,
}

fn handle_favorites(
    userdata: &mut UserData,
    to_add: &Vec<String>,
    to_remove: &Vec<String>,
) -> bool {
    let mut changed = false;
    for favorite in to_add {
        changed |= userdata.favorites.insert(dealer_from_string(favorite))
    }
    for favorite in to_remove {
        changed |= userdata.favorites.remove(&dealer_from_string(favorite))
    }
    changed
}

async fn handle_search(
    mut userdata: UserData,
    search_items: Vec<String>,
    favorites_changed: bool,
) -> Vec<Offer> {
    let mut offers = vec![];
    match search_items.len() {
        1.. => {
            for search in search_items {
                let mut temp = retrieve_offers(&mut userdata, favorites_changed).await;
                temp.retain(|offer| offer.name.to_lowercase().contains(search.trim()));
                offers.append(&mut temp);
            }
        }
        0 => {
            println!("No search term provided, not filtering offers...");
            offers = retrieve_offers(&mut userdata, favorites_changed).await;
        }
        _ => panic!("shouldn't be possible"),
    }
    offers.sort_by(|a, b| {
        (a.name.as_str(), a.dealer.as_str())
            .partial_cmp(&(b.name.as_str(), b.dealer.as_str()))
            .expect("couldn't compare Offers in sorting")
    });
    offers.dedup();
    println!("Amount of offers: {}", offers.len());
    offers
}

async fn retrieve_offers(userdata: &mut UserData, favorites_changed: bool) -> Vec<Offer> {
    let mut offers = vec![];
    let today = Utc::now().date_naive();
    if favorites_changed || today.signed_duration_since(userdata.rerun_date).num_days() > 0 {
        for dealer in &userdata.favorites {
            offers.append(
                &mut retrieve_offers_from_remote(dealer)
                    .await
                    .expect("Failed to retrieve remote offers"),
            );
        }
        cache_retrieved_offers(&offers, userdata).expect("Was unable to cache offers");
    } else {
        offers = retrieve_cached_offers().expect("Was unable to receive offers from cache");
    }
    offers
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
        "./data/offer_cache.json",
        serde_json::to_string(offers).expect("Could not write \"cached offers\""),
    )?;
    println!("WRITTEN offer cache");
    userdata.rerun_date = match offers.iter().map(|o| o.run_till).min() {
        Some(date) => date,
        None => userdata.rerun_date,
    };
    fs::write(
        "./data/userdata.json",
        serde_json::to_string(userdata).unwrap(),
    )?;
    println!("WRITTEN date");
    Ok(())
}

fn retrieve_cached_offers() -> Option<Vec<Offer>> {
    // TODO Why can't I retrieve from subdir? such as ./data/cached_offers.json
    serde_json::from_str(
        &fs::read_to_string("data/offer_cache.json").expect("Cannot open file: cached_offers.json"),
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
