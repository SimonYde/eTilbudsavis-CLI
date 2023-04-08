use super::Offer;
use serde::Deserialize;

pub fn deserialize_dealer_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Dealer {
        name: String,
    }

    let helper = Dealer::deserialize(deserializer)?;
    Ok(helper.name)
}

#[derive(Deserialize)]
pub struct OfferWrapper {
    offer: Outer,
}
#[derive(Deserialize)]
struct Outer {
    #[serde(rename = "heading")]
    name: String,
    pricing: Pricing,
    run_from: String,
    run_till: String,
    quantity: Quantity,
}

#[derive(Deserialize)]
struct Pricing {
    price: f64,
}

#[derive(Deserialize)]
struct Quantity {
    unit: Unit,
    pieces: Pieces,
    size: Size,
}

#[derive(Deserialize)]
struct Size {
    from: f64,
    to: f64,
}

#[derive(Deserialize)]
struct Pieces {
    from: u32,
    to: u32,
}

#[derive(Deserialize)]
struct Unit {
    si: SI,
}

#[derive(Deserialize)]
struct SI {
    symbol: String,
    factor: f64,
}
pub fn deserialize_offer(offer_wrapper: OfferWrapper, dealer_name: &str) -> Offer {
    let offer = &offer_wrapper.offer;
    let factor = &offer.quantity.unit.si.factor;
    let pieces = &offer.quantity.pieces;
    let size = &offer.quantity.size;
    Offer {
        name: offer.name.to_owned(),
        price: offer.pricing.price,
        min_amount: pieces.from,
        max_amount: pieces.to,
        min_size: size.from * factor,
        max_size: size.to * factor,
        unit: offer.quantity.unit.si.symbol.to_owned(),
        cost_per_unit: offer.pricing.price / (size.to * factor) / pieces.to as f64,
        dealer: dealer_name.to_owned(),
        run_from: chrono::NaiveDate::parse_from_str(
            offer.run_from.split('T').next().unwrap(),
            "%Y-%m-%d",
        )
        .expect("failed to format NaiveDate from API date"),
        run_till: chrono::NaiveDate::parse_from_str(
            offer.run_till.split('T').next().unwrap(),
            "%Y-%m-%d",
        )
        .expect("failed to format NaiveDate from API date"),
    }
}
