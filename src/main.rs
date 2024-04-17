mod requests;
use clap::Parser;

use crate::requests::{
    dealer::Dealer,
    offer::{retrieve_offers, Offer},
    userdata::UserData,
};
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let runtime = std::time::Instant::now();
    let args = Args::parse();
    args.run().await;

    println!("{:?}", runtime.elapsed());
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(use_value_delimiter = true, value_delimiter = ',', short, long)]
    add_dealers_to_favorites: Vec<String>,
    #[arg(use_value_delimiter = true, value_delimiter = ',', short, long)]
    remove_dealers_from_favorites: Vec<String>,
    #[arg(use_value_delimiter = true, value_delimiter = ',')]
    search: Vec<String>,
    #[arg(short, long)]
    favorites: bool,
    #[arg(short, long)]
    known_dealers: bool,
    #[arg(short, long, default_value_t = false)]
    print: bool,
    #[arg(short, long)]
    json: bool,
}

impl Args {
    async fn run(self) {
        let args = Args::parse();
        let mut userdata = requests::userdata::get_userdata();
        let favorites_changed = handle_favorites(
            &mut userdata,
            &args.add_dealers_to_favorites,
            &args.remove_dealers_from_favorites,
        );
        let mut offers = handle_search(&mut userdata, args.search, favorites_changed).await;
        offers.sort_by(|a, b| a.cost_per_unit.total_cmp(&b.cost_per_unit).reverse());
        if args.print {
            offers.iter().for_each(|offer| println!("{:#?}", offer))
        }
        if args.favorites {
            println!("Favorites:");
            userdata
                .favorites
                .iter()
                .for_each(|dealer| println!(" - {:#?}", dealer));
        }
        if args.known_dealers {
            println!("Known Dealers:");
            Dealer::list_known_dealers();
        }
    }
}

fn handle_favorites(
    userdata: &mut UserData,
    to_add: &Vec<String>,
    to_remove: &Vec<String>,
) -> bool {
    let mut changed = false;
    for favorite in to_add {
        if let Ok(dealer) = Dealer::from_str(favorite) {
            changed |= userdata.favorites.insert(dealer)
        } else {
            println!("Attempted to add unknown dealer `{}`.", favorite);
            Dealer::list_known_dealers();
        }
    }
    for favorite in to_remove {
        if let Ok(dealer) = Dealer::from_str(favorite) {
            changed |= userdata.favorites.remove(&dealer);
        } else {
            println!("Attempted to remove unknown dealer `{}`.", favorite);
            Dealer::list_known_dealers();
        }
    }
    if changed {
        userdata.save();
    }
    changed
}

async fn handle_search(
    userdata: &mut UserData,
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
            offers.sort_unstable_by(|a, b| (&a.name, &a.dealer).cmp(&(&b.name, &b.dealer)));
            offers.dedup();
        }
        0 => {
            println!("No search term provided, not filtering offers...");
            offers = retrieve_offers(userdata, favorites_changed).await;
        }
    }
    println!("Amount of offers: {}", offers.len());
    offers
}
