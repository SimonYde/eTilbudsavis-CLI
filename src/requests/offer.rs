use chrono::NaiveDate;
use comfy_table::{Cell, CellAlignment};
use serde::{Deserialize, Serialize};

use super::dealer::Dealer;

#[derive(Debug, Deserialize, Serialize, PartialOrd)]
pub(crate) struct Offer {
    pub id: String,
    pub name: String,
    pub dealer: Dealer,
    pub price: f64,
    pub cost_per_unit: f64,
    pub unit: String,
    pub min_size: f64,
    pub max_size: f64,
    pub min_amount: u32,
    pub max_amount: u32,
    pub run_from: NaiveDate,
    pub run_till: NaiveDate,
}

pub fn sort_by_cost(a: &Offer, b: &Offer) -> std::cmp::Ordering {
    a.cost_per_unit.total_cmp(&b.cost_per_unit).reverse()
}

impl PartialEq for Offer {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            || (self.dealer == other.dealer
                && self.name == other.name
                && self.run_from == other.run_from
                && self.run_till == other.run_till)
    }
}

impl Eq for Offer {}

impl std::fmt::Display for Offer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let offer_str = format!(
            "{} - {}: {} - {}: {} kr. - {:.2} kr/{}",
            self.run_from.format("%d/%m"),
            self.run_till.format("%d/%m"),
            self.dealer,
            self.name,
            self.price,
            self.cost_per_unit,
            self.unit
        );
        write!(f, "{}", offer_str)?;
        Ok(())
    }
}

impl Offer {
    pub(crate) fn to_table_entry(&self) -> Vec<Cell> {
        let unit = &self.unit;
        let period = format!(
            "{}\n  ↓  \n{}",
            self.run_from.format("%d/%m"),
            self.run_till.format("%d/%m")
        );
        let cost_per_unit = format!("{:.2} kr/{}", self.cost_per_unit, unit);
        let price = format!("{:.2} kr", self.price);
        let count = if self.min_amount == self.max_amount {
            format!("{}", self.min_amount)
        } else {
            format!("{}-{}", self.min_amount, self.max_amount)
        };

        let min_size_is_decimal = self.min_size - self.min_size.trunc() > 0.01;
        let max_size_is_decimal = self.max_size - self.max_size.trunc() > 0.01;
        let max_size_equals_min_size = self.max_size - self.min_size < 0.001;

        let min_size = if min_size_is_decimal {
            format!("{:.3}", self.min_size)
        } else {
            format!("{}", self.min_size)
        };
        let max_size = if max_size_is_decimal {
            format!("{:.3}", self.max_size)
        } else {
            format!("{}", self.max_size)
        };

        let weight = if max_size_equals_min_size {
            format!("{} {}", min_size, unit)
        } else {
            format!("{}-{} {}", min_size, max_size, unit)
        };

        vec![
            Cell::new(period),
            Cell::new(self.dealer.to_string()),
            Cell::new(self.name.to_string()),
            Cell::new(count),
            Cell::new(price).set_alignment(CellAlignment::Right),
            Cell::new(cost_per_unit).set_alignment(CellAlignment::Right),
            Cell::new(weight).set_alignment(CellAlignment::Right),
        ]
    }
}
