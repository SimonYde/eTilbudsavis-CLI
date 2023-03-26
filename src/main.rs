use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Response,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    //let args: Vec<String> = env::args().collect();
    let netto = Dealer {
        id: "9ba51".to_string(),
        name: "Netto".to_string(),
    };
    println!("{:?}", netto);
    println!(
        "{:?}",
        retrieve_catalog(&netto)
            .await
            .unwrap()
            .into_iter()
            .last()
            .unwrap()
    );
}

#[derive(Debug)]
struct Offer {
    id: String,
    price: u32,
    amount: u32,
    unit: String,
    start_date: String,
    end_date: String,
}

#[derive(Debug)]
struct Dealer {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Catalog {
    id: String,
    run_from: String,
    run_till: String,
    dealer_id: String,
    offer_count: u32,
}

async fn retrieve_catalog(dealer: &Dealer) -> Option<Vec<Offer>> {
    let client = reqwest::Client::new();
    let catalog_response = client
        .get("https://squid-api.tjek.com/v2/catalogs")
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .query(&[("dealer_ids", dealer.id.as_str())])
        .send()
        .await
        .unwrap();

    let catalogs = match catalog_response.status() {
        reqwest::StatusCode::OK => match catalog_response.json::<Vec<Catalog>>().await {
            Ok(parsed) => {
                println!("success!\n{:?}", parsed);
                parsed
            }
            Err(_) => panic!("Tried parsing JSON that wasn't a Catalog"),
        },
        _ => panic!("didn't get a valid response, perhaps there is no connection?"),
    };
    let temp_offer = Offer {
        id: "test".to_string(),
        price: 12345,
        amount: 1,
        unit: "kg".to_string(),
        start_date: "2023-03-04".to_string(),
        end_date: "2023-04-04".to_string(),
    };
    Some(vec![temp_offer])
}
