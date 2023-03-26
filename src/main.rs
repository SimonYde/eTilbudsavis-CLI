use reqwest::Client;
use tokio::main;

#[tokio::main]
async fn main() {
    //let args: Vec<String> = env::args().collect();
    let netto = Vendor {
        id: "9ba51".to_string(),
        name: "Netto".to_string(),
    };
    println!("{} has id: {}", netto.name, netto.id);
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

struct Vendor {
    id: String,
    name: String,
}

async fn retrieve_catalog(vendor: &Vendor) -> Option<Vec<Offer>> {
    let client = reqwest::Client::new();
    let request = client
        .get("https://squid-api.tjek.com/v2/catalogs")
        .query(&[
            ("dealer_ids", vendor.id.as_str()),
            ("order_by", "created"),
            ("limit", "6"),
        ]);
    println!("{:?}", request);
    let response = request.send().await.unwrap();

    match response.status() {
        reqwest::StatusCode::OK => {
            let response_text = response.text().await;
            println!("success! {:?}", response_text);

            // let json_text = response_text.unwrap();
        }
        _ => {
            panic!("didn't get a valid response, perhaps there is no connection?")
        }
    }
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
