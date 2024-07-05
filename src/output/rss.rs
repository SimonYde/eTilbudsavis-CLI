use crate::Offer;
use std::fmt::Write;

/// Create rss feed with list of offers
pub fn offers_as_rss(offers: &[Offer]) -> Result<String, std::fmt::Error> {
    let mut output = String::new();

    write!(output, "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n")?;
    write!(output, "<rss version=\"2.0\">\n")?;
    write!(output, "\t<channel>\n")?;

    for offer in offers {
        write!(output, "\t<item>\n")?;
        write!(
            output, "\t\t<title>[{}] [{}] {}</title>\n", offer.dealer, offer.price, offer.name
        )?;
        write!(output, "\t\t<pubDate>{}</pubDate>\n", offer.run_from.to_string())?;
        write!(output, "\t</item>\n")?;
    }

    write!(output, "\t</channel>\n")?;
    write!(output, "</rss>")?;

    return Ok(output);
}
