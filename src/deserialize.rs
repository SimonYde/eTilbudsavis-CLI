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
pub fn deserialize_offer_price<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Pricing {
        price: f64,
    }

    let helper = Pricing::deserialize(deserializer)?;
    Ok(helper.price)
}
