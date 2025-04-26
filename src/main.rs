mod output;
mod requests;

use crate::requests::{dealer::Dealer, offer::Offer, userdata::UserData};
use clap::{Command, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Generator, Shell, generate};
use clap_complete_nushell::Nushell;
use output::OutputFormat;
use requests::offer::sort_by_cost;
use std::process::exit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Shells {
    Bash,
    Fish,
    Nushell,
    Zsh,
}

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
    #[arg(short, long)]
    dealer: bool,

    #[arg(long = "generate", value_enum)]
    generator: Option<Shells>,

    #[command(subcommand)]
    favorites: Option<FavoriteCommands>,
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

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut std::io::stdout(),
    );
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let mut userdata = UserData::from_cache().unwrap_or_default();

    if let Some(shell) = args.generator {
        let mut cmd = Cli::command();
        eprintln!("Generating completion file for {shell:?}...");
        let generator = match shell {
            Shells::Bash => Shell::Bash,
            Shells::Fish => Shell::Fish,
            Shells::Zsh => Shell::Zsh,
            Shells::Nushell => {
                print_completions(Nushell, &mut cmd);
                exit(0)
            }
        };
        print_completions(generator, &mut cmd);
        exit(0);
    }

    match args.favorites {
        Some(FavoriteCommands::Add { dealers }) => userdata.add_favorites(&dealers),
        Some(FavoriteCommands::Remove { dealers }) => userdata.remove_favorites(&dealers),
        Some(FavoriteCommands::Dealers) => {
            Dealer::list_known_dealers(args.format);
            exit(0);
        }
        Some(FavoriteCommands::Favorites) => {
            userdata.print_favorites(args.format);
            exit(0);
        }
        None => (),
    };

    let mut offers = userdata.search(&args.search, args.dealer).await;
    offers.sort_unstable_by(|a, b| sort_by_cost(a, b));

    match args.format {
        Some(format) => output::print_offers(offers, &format),
        None => println!("Amount of offers: {}", offers.len()),
    }
}
