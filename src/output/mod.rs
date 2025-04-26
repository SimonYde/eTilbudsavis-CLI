mod rss;
mod table;

use crate::Offer;
use clap::ValueEnum;

/// Format to print offers in
#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum OutputFormat {
    Json,
    Rss,
    Table,
}

/// Print offers in the specified format
pub fn print_offers(offers: Vec<&Offer>, format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&offers).expect("dude what?"));
        }
        OutputFormat::Rss => {
            let rss = rss::offers_as_rss(offers).expect("Could not create rss feed");
            println!("{}", rss);
        }
        OutputFormat::Table => table::print_as_table(offers),
    }
}
