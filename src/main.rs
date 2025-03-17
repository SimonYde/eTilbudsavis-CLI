mod output;
mod requests;
use clap::{Parser, Subcommand, ValueEnum};

use crate::requests::{dealer::Dealer, offer::Offer, userdata::UserData};
use requests::offer::sort_by_cost;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(
    author, version, about = "A CLI interface for the eTilbudsavis API.", long_about = None
)]
struct Cli {
    search: Vec<String>,

    /// The desired output format.
    #[arg(short, long)]
    format: Option<OutputFormat>,

    /// Search by dealer.
    #[arg(short)]
    dealer: bool,

    #[command(subcommand)]
    favorites: Option<FavoriteCommands>,
}

impl Cli {}

/// Format to print offers in
#[derive(Debug, ValueEnum, Clone, Copy)]
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

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let mut userdata = UserData::from_cache().unwrap_or_default();

    match args.favorites {
        Some(FavoriteCommands::Add { dealers }) => userdata.add_favorites(&dealers),
        Some(FavoriteCommands::Remove { dealers }) => userdata.remove_favorites(&dealers),
        Some(FavoriteCommands::Dealers) => {
            Dealer::list_known_dealers(args.format);
            exit(0);
        }
        Some(FavoriteCommands::Favorites) => {
            userdata.print_favorites();
            exit(0);
        }
        None => (),
    };

    let mut offers = userdata.search(&args.search, args.dealer).await;
    offers.sort_unstable_by(|a, b| sort_by_cost(a, b));

    match args.format {
        Some(format) => output::print_offers(&offers, &format),
        None => println!("Amount of offers: {}", offers.len()),
    }
}
