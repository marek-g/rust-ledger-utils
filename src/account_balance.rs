use crate::prices::{Prices, PricesError};
use chrono::NaiveDate;
use ledger_parser::*;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use std::collections::HashMap;
use std::fmt;
use std::ops::AddAssign;
use std::ops::SubAssign;

/// Balance of an single account.
///
/// Maps commodity names to amounts.
#[derive(Clone)]
pub struct AccountBalance {
    pub amounts: HashMap<String, Amount>,
}

impl Default for AccountBalance {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountBalance {
    pub fn new() -> AccountBalance {
        AccountBalance {
            amounts: HashMap::new(),
        }
    }

    pub fn value_in_commodity(
        &self,
        commodity_name: &str,
        date: NaiveDate,
        prices: &Prices,
    ) -> Result<Decimal, PricesError> {
        let mut result = Decimal::new(0, 0);
        for amount in self.amounts.values() {
            if amount.commodity.name == commodity_name {
                result += amount.quantity;
            } else {
                result += prices.convert(
                    amount.quantity,
                    &amount.commodity.name,
                    commodity_name,
                    date,
                )?;
            }
        }
        Ok(result)
    }

    pub fn value_in_commodity_rounded(
        &self,
        commodity_name: &str,
        decimal_points: u32,
        date: NaiveDate,
        prices: &Prices,
    ) -> Decimal {
        let assets_value = self.value_in_commodity(commodity_name, date, prices);
        if let Ok(value) = assets_value {
            value.round_dp_with_strategy(decimal_points, RoundingStrategy::MidpointAwayFromZero)
        } else {
            panic!("{:?}", assets_value);
        }
    }

    pub fn is_zero(&self) -> bool {
        self.amounts
            .iter()
            .all(|(_, amount)| amount.quantity == Decimal::ZERO)
    }

    fn remove_empties(&mut self) {
        let empties: Vec<String> = self
            .amounts
            .iter()
            .filter(|(_, amount)| amount.quantity == Decimal::ZERO)
            .map(|(k, _)| k.clone())
            .collect();
        for empty in empties {
            self.amounts.remove(&empty);
        }
    }
}

impl<'a> AddAssign<&'a AccountBalance> for AccountBalance {
    fn add_assign(&mut self, other: &'a AccountBalance) {
        for (currrency_name, amount) in &other.amounts {
            self.amounts
                .entry(currrency_name.clone())
                .and_modify(|a| a.quantity += amount.quantity)
                .or_insert_with(|| amount.clone());
        }
        self.remove_empties();
    }
}

impl<'a> AddAssign<&'a ledger_parser::Amount> for AccountBalance {
    fn add_assign(&mut self, amount: &'a ledger_parser::Amount) {
        self.amounts
            .entry(amount.commodity.name.clone())
            .and_modify(|a| a.quantity += amount.quantity)
            .or_insert_with(|| amount.clone());
        self.remove_empties();
    }
}

impl<'a> SubAssign<&'a AccountBalance> for AccountBalance {
    fn sub_assign(&mut self, other: &'a AccountBalance) {
        for (currrency_name, amount) in &other.amounts {
            self.amounts
                .entry(currrency_name.clone())
                .and_modify(|a| a.quantity -= amount.quantity)
                .or_insert_with(|| amount.clone());
        }
        self.remove_empties();
    }
}

impl<'a> SubAssign<&'a ledger_parser::Amount> for AccountBalance {
    fn sub_assign(&mut self, amount: &'a ledger_parser::Amount) {
        self.amounts
            .entry(amount.commodity.name.clone())
            .and_modify(|a| a.quantity -= amount.quantity)
            .or_insert_with(|| amount.clone());
        self.remove_empties();
    }
}

impl fmt::Debug for AccountBalance {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut values: Vec<Amount> = self.amounts.values().cloned().collect();
        values.sort_by(|a, b| a.commodity.name.partial_cmp(&b.commodity.name).unwrap());
        write!(f, "{:?}", values)
    }
}
