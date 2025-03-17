mod rss;

use crate::{Offer, OutputFormat};
use comfy_table::{ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};

/// Print offers in the specified format
pub fn print_offers(offers: &[&Offer], format: &OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string(&offers).expect("dude what?"));
        }
        OutputFormat::Rss => {
            let rss = rss::offers_as_rss(offers).expect("Could not create rss feed");
            println!("{}", rss);
        }
        OutputFormat::Table => print_as_table(offers),
    }
}

/// Print offers as a table
fn print_as_table(offers: &[&Offer]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(vec![
            "Period",
            "Dealer",
            "Product",
            "Count",
            "Price",
            "Cost/unit",
            "Weight",
        ]);

    for offer in offers.iter() {
        table.add_row(offer.to_table_entry());
    }

    println!("{}", table);
}
