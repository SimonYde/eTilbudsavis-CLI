use chrono::{NaiveDate, Utc};
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::str::FromStr;
// use tokio_stream::StreamExt;

mod dealer;
use crate::dealer::*;
mod deserialize;
use crate::deserialize::*;

#[tokio::main]
async fn main() {
    let runtime = std::time::Instant::now();
    let args = Args::parse();
    let mut userdata = get_userdata();
    let favorites_changed =
        handle_favorites(&mut userdata, &args.add_favorites, &args.remove_favorites);
    let mut offers = handle_search(&userdata, args.search, favorites_changed).await;
    offers.sort_by(|a, b| a.cost_per_unit.total_cmp(&b.cost_per_unit).reverse());
    if args.print {
        offers.iter().for_each(|offer| println!("{:?}", offer))
    }
    println!("{:?}", runtime.elapsed());
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

fn get_userdata() -> UserData {
    match fs::read_to_string("data/userdata.json") {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => UserData::default(),
    }
}

#[derive(Serialize, Deserialize)]
struct UserData {
    favorites: HashSet<Dealer>,
}

impl Default for UserData {
    fn default() -> Self {
        println!(
            "First run, or userdata was modified outside of running of this program.\nReinitialising userdata with no favorites..."
        );
        UserData {
            favorites: HashSet::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Offer {
    // id: String,
    name: String,
    dealer: String,
    price: f64,
    cost_per_unit: f64,
    unit: String,
    min_size: f64,
    max_size: f64,
    min_amount: u32,
    max_amount: u32,
    run_from: NaiveDate,
    run_till: NaiveDate,
}

impl Default for Offer {
    fn default() -> Self {
        Offer {
            name: String::default(),
            dealer: String::default(),
            price: f64::default(),
            cost_per_unit: f64::default(),
            unit: String::default(),
            min_size: f64::default(),
            max_size: f64::default(),
            min_amount: u32::default(),
            max_amount: u32::default(),
            run_from: Utc::now().date_naive(),
            run_till: Utc::now().date_naive(),
        }
    }
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
        changed |= userdata.favorites.insert(
            Dealer::from_str(favorite).expect("Was given a name that does match a known dealer."),
        )
    }
    for favorite in to_remove {
        changed |= userdata.favorites.remove(
            &Dealer::from_str(favorite)
                .expect("Was given a name that didn't match a known dealer."),
        )
    }
    if changed {
        fs::write(
            "./data/userdata.json",
            serde_json::to_string(userdata).unwrap(),
        )
        .unwrap();
        println!("WRITTEN userdata");
    }
    changed
}

async fn handle_search(
    userdata: &UserData,
    search_items: Vec<String>,
    favorites_changed: bool,
) -> Vec<Offer> {
    let mut offers = Vec::new();
    match search_items.len() {
        1.. => {
            for search in search_items {
                let mut temp = retrieve_offers(userdata, favorites_changed).await;
                match Dealer::from_str(&search) {
                    Ok(_) => {
                        temp.retain(|offer| offer.dealer.to_lowercase() == search.to_lowercase())
                    }
                    Err(_) => {
                        temp.retain(|offer| offer.name.to_lowercase().contains(search.trim()))
                    }
                }
                offers.append(&mut temp);
            }
            offers.sort_by(|a, b| {
                (a.name.as_str(), a.dealer.as_str()).cmp(&(b.name.as_str(), b.dealer.as_str()))
            });
            offers.dedup();
        }
        0 => {
            println!("No search term provided, not filtering offers...");
            offers = retrieve_offers(userdata, favorites_changed).await;
        }
        _ => unreachable!(),
    }
    println!("Amount of offers: {}", offers.len());
    offers
}

async fn retrieve_offers(userdata: &UserData, favorites_changed: bool) -> Vec<Offer> {
    let today = Utc::now().date_naive();
    match retrieve_cached_offers() {
        Ok(offers) => {
            if favorites_changed || today > offers.first().unwrap_or(&Offer::default()).run_till {
                return retrieve_offers_from_remote(userdata)
                    .await
                    .unwrap_or_default();
            }
            offers
        }
        Err(_) => retrieve_offers_from_remote(userdata)
            .await
            .unwrap_or_default(),
    }
}

async fn retrieve_offers_from_remote(userdata: &UserData) -> Option<Vec<Offer>> {
    let mut offers = Vec::new();
    for dealer in &userdata.favorites {
        offers.append(&mut remote_offers_for_dealer(dealer).await.unwrap_or_default());
    }
    cache_retrieved_offers(&mut offers).expect("Was unable to cache offers");
    Some(offers)
}

async fn remote_offers_for_dealer(dealer: &Dealer) -> Option<Vec<Offer>> {
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

async fn retrieve_catalogs_from_dealer(dealer: &Dealer, client: &Client) -> Option<Vec<Catalog>> {
    let catalog_response = client
        .get("https://squid-api.tjek.com/v2/catalogs")
        .query(&[("dealer_ids", dealer.id())])
        .send()
        .await
        .ok()?;
    match catalog_response.status() {
        reqwest::StatusCode::OK => catalog_response.json::<Vec<Catalog>>().await.ok(),
        _ => {
            println!(
                "Did not succesfully access API, no connection? StatusCode: {}",
                catalog_response.status()
            );
            Some(Vec::new())
        }
    }
}

async fn retrieve_offers_from_catalog(
    catalog: Catalog,
    client: &Client,
) -> Result<Vec<Offer>, reqwest::Error> {
    let offers = client
        .get(format!(
            "https://squid-api.tjek.com/v2/catalogs/{}/hotspots",
            catalog.id.as_str()
        ))
        .send()
        .await?
        .json::<Vec<OfferWrapper>>()
        .await?
        .into_iter()
        .map(|ow| deserialize_offer(ow, &catalog.dealer))
        .collect();
    Ok(offers)
}

fn cache_retrieved_offers(offers: &mut Vec<Offer>) -> std::io::Result<()> {
    offers.sort_by(|a, b| a.run_till.cmp(&b.run_till));
    fs::write(
        "./data/offer_cache.json",
        serde_json::to_string(offers).expect("Could not write \"cached offers\""),
    )?;
    println!("WRITTEN offer cache");
    Ok(())
}

fn retrieve_cached_offers() -> Result<Vec<Offer>, serde_json::Error> {
    let offer_cache_str = &match fs::read_to_string("data/offer_cache.json") {
        Ok(cache) => cache,
        Err(err) => err.to_string(),
    };
    serde_json::from_str(offer_cache_str)
}
