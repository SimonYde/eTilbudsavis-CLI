use crate::Offer;
use std::fmt::Write;

/// Create RSS feed with list of offers
pub fn offers_as_rss(offers: &[&Offer]) -> Result<String, std::fmt::Error> {
    let mut output = String::new();

    writeln!(output, r#"<?xml version="1.0" encoding="UTF-8" ?>"#)?;
    writeln!(output, r#"<rss version="2.0">"#)?;
    writeln!(output, "\t<channel>")?;

    for offer in offers {
        writeln!(output, "\t<item>")?;
        writeln!(
            output,
            "\t\t<title>[{}] [{}] {}</title>",
            offer.dealer, offer.price, offer.name
        )?;
        writeln!(output, "\t\t<pubDate>{}</pubDate>", offer.run_from)?;
        writeln!(output, "\t</item>")?;
    }

    writeln!(output, "\t</channel>")?;
    write!(output, "</rss>")?;

    Ok(output)
}
