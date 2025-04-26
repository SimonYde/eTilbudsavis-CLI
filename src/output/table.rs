use crate::Offer;
use comfy_table::{ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL};

/// Print offers as a table
pub fn print_as_table(offers: Vec<&Offer>) {
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
