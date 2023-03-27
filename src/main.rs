use reqwest::header::{ACCEPT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[tokio::main]
async fn main() {
    println!(
        "{:?}",
        retrieve_offers_from_catalogs(&Dealer::Netto)
            .await
            .unwrap()
            .into_iter()
            .take(1)
    );
}

#[derive(Debug)]
enum Dealer {
    Rema1000,
    Netto,
}

impl Dealer {
    fn id(&self) -> String {
        match self {
            Dealer::Rema1000 => String::from("11deC"),
            Dealer::Netto => String::from("9ba51"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Offer {
    id: String,
    name: String,
    price: u32,
    min_amount: u32,
    max_amount: u32,
    min_size: f64,
    max_size: f64,
    unit: String,
    start_date: String,
    end_date: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Catalog {
    id: String,
    run_from: String,
    run_till: String,
    dealer_id: String,
    offer_count: u32,
}

async fn retrieve_offers_from_catalogs(dealer: &Dealer) -> Option<Vec<Offer>> {
    let client = reqwest::Client::new();
    let catalog_response = client
        .get("https://squid-api.tjek.com/v2/catalogs")
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .query(&[("dealer_ids", dealer.id().as_str())])
        .send()
        .await
        .ok()?;

    let catalogs = if let reqwest::StatusCode::OK = catalog_response.status() {
        catalog_response.json::<Vec<Catalog>>().await.ok()?
    } else {
        panic!("didn't get a valid response, perhaps there is no connection?")
    };

    let catalog_ids: Vec<String> = catalogs.into_iter().map(|c| c.id).collect();
    let mut offers: Vec<Offer> = vec![];
    for id in catalog_ids {
        let offers_response = client
            .get(format!(
                "https://squid-api.tjek.com/v2/catalogs/{}/hotspots",
                id.as_str()
            ))
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json")
            .send()
            .await
            .ok()?;
        let parsed = offers_response.json::<Vec<Value>>().await.unwrap();
        let mut temp = parsed.into_iter().filter_map(create_offer).collect();
        offers.append(&mut temp);
    }

    Some(offers)
}

fn create_offer(x: Value) -> Option<Offer> {
    let offer = &x["offer"];
    let quantity = &offer["quantity"];
    let factor = quantity["unit"]["si"]["factor"].as_f64()?;
    Some(Offer {
        id: offer["id"].to_string(),
        name: offer["heading"].to_string(),
        price: offer["pricing"]["price"].as_u64()? as u32,
        min_amount: quantity["pieces"]["from"].as_u64()? as u32,
        max_amount: quantity["pieces"]["to"].as_u64()? as u32,
        min_size: quantity["size"]["from"].as_f64()? * factor,
        max_size: quantity["size"]["to"].as_f64()? * factor,
        unit: quantity["unit"]["si"]["symbol"].to_string(),
        start_date: offer["run_from"].to_string().split('T').next()?.to_string(),
        end_date: offer["run_till"].to_string().split('T').next()?.to_string(),
    })
}
