mod requests;
mod output;
use clap::{Parser, Subcommand, ValueEnum};
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Table};

use crate::requests::{
    dealer::Dealer,
    offer::{retrieve_offers, Offer},
    userdata,
    userdata::UserData,
};
use std::{borrow::Cow, process::exit, str::FromStr};

#[tokio::main]
async fn main() {
    let runtime = std::time::Instant::now();
    let args = Cli::parse();
    run(args).await;
    dbg!(runtime.elapsed());
}

async fn run(args: Cli) {
    let mut userdata = userdata::get_userdata();

    let favorites_changed = match args.favorites {
        Some(FavoriteCommands::Add { dealers }) => userdata.add_favorites(&dealers),
        Some(FavoriteCommands::Remove { dealers }) => userdata.remove_favorites(&dealers),
        Some(FavoriteCommands::Dealers) => {
            Dealer::list_known_dealers();
            exit(0);
        }
        Some(FavoriteCommands::Favorites) => {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.apply_modifier(UTF8_ROUND_CORNERS);
            table.set_header(vec!["Favorites"]);
            for favorite in userdata.favorites {
                table.add_row(vec![favorite]);
            }
            println!("{}", table);
            exit(0);
        }
        None => false,
    };

    let mut offers =
        handle_search(&mut userdata, &args.search, favorites_changed, args.dealer).await;
    offers.sort_unstable_by(|a, b| a.cost_per_unit.total_cmp(&b.cost_per_unit).reverse());

    match args.format {
        Some(format) => output::print_offers(&offers, &format),
        None => println!("Amount of offers: {}", offers.len()),
    }
}

#[derive(Parser, Debug)]
#[command(
    author, version, about = "A CLI interface for the eTilbudsavis API.", long_about = None
)]
struct Cli {
    search: Vec<Cow<'static, str>>,

    /// Format to print offers in
    #[arg(short, long)]
    format: Option<OutputFormat>,

    /// Search by dealer
    #[arg(short)]
    dealer: bool,

    #[command(subcommand)]
    favorites: Option<FavoriteCommands>,
}

/// Format to print offers in
#[derive(Debug, ValueEnum, Clone)]
pub enum OutputFormat {
    Json,
    Rss,
    Table,
}

#[derive(Subcommand, Debug)]
#[command(author, version, about, long_about = None)]
enum FavoriteCommands {
    #[command(about = "Add a dealer to favorites")]
    Add { dealers: Vec<Dealer> },
    #[command(about = "Remove a dealer from favorites")]
    Remove { dealers: Vec<Dealer> },
    #[command(about = "List available dealers")]
    Dealers,
    #[command(about = "List currently set favorites")]
    Favorites,
}

async fn handle_search(
    userdata: &mut UserData,
    search_items: &Vec<Cow<'_, str>>,
    favorites_changed: bool,
    search_by_dealer: bool,
) -> Vec<Offer> {
    if !search_items.is_empty() {
        let mut offers = Vec::new();
        for search in search_items {
            let mut temp = retrieve_offers(userdata, favorites_changed).await;
            if search_by_dealer {
                if let Ok(dealer) = Dealer::from_str(search) {
                    temp.retain(|offer| offer.dealer == dealer);
                } else {
                    println!("Search term did not match any known dealers: {search}");
                    Dealer::list_known_dealers();
                }
            } else {
                temp.retain(|offer| offer.name.to_lowercase().contains(search.trim()))
            }
            offers.extend(temp);
        }
        offers.sort_unstable_by(|a, b| (&a.name, &a.dealer).cmp(&(&b.name, &b.dealer)));
        offers.dedup();
        offers
    } else {
        retrieve_offers(userdata, favorites_changed).await
    }
}
