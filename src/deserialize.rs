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
pub struct Outer {
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
pub fn deserialize_offer(outer: Outer, dealer_name: &str) -> Offer {
    Offer {
        name: outer.name,
        price: outer.pricing.price,
        min_amount: outer.quantity.pieces.from,
        max_amount: outer.quantity.pieces.to,
        min_size: outer.quantity.size.from * outer.quantity.unit.si.factor,
        max_size: outer.quantity.size.to * outer.quantity.unit.si.factor,
        run_from: outer.run_from.split('T').next().unwrap().to_owned(),
        run_till: outer.run_till.split('T').next().unwrap().to_owned(),
        unit: outer.quantity.unit.si.symbol,
        cost_per_unit: outer.pricing.price
            / (outer.quantity.size.to * outer.quantity.unit.si.factor)
            / outer.quantity.pieces.to as f64,
        dealer: dealer_name.to_owned(),
    }
}
