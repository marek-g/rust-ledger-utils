use chrono::NaiveDate;
use ledger_parser::*;
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::collections::HashMap;

#[derive(Debug)]
pub enum PricesError {
    NoSuchCommoditiesPair(CommoditiesPair),
    DateTooEarly(NaiveDate),
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct CommoditiesPair {
    pub src_commodity_name: String,
    pub dst_commodity_name: String,
}

impl CommoditiesPair {
    pub fn new(src_commodity_name: &str, dst_commodity_name: &str) -> CommoditiesPair {
        CommoditiesPair {
            src_commodity_name: src_commodity_name.to_string(),
            dst_commodity_name: dst_commodity_name.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct RatesTable {
    pub table: BTreeMap<NaiveDate, Decimal>,
}

impl RatesTable {
    fn new() -> RatesTable {
        RatesTable {
            table: BTreeMap::new(),
        }
    }

    fn get_rate(&self, date: NaiveDate) -> Result<Decimal, PricesError> {
        let mut rate: Option<Decimal> = None;
        for (key, value) in self.table.iter() {
            if *key <= date {
                rate = Some(*value)
            } else {
                break;
            }
        }
        rate.ok_or(PricesError::DateTooEarly(date))
    }
}

#[derive(Debug)]
pub struct Prices {
    pub rates: HashMap<CommoditiesPair, RatesTable>,
}

impl Prices {
    pub fn load(ledger: &Ledger) -> Prices {
        let mut result = Prices {
            rates: HashMap::new(),
        };

        result.add_prices(&get_commodity_prices(ledger));
        result.add_prices(&get_prices_from_transactions(ledger));

        result
    }

    pub fn convert(
        &self,
        amount: Decimal,
        src_commodity_name: &str,
        dst_commodity_name: &str,
        date: NaiveDate,
    ) -> Result<Decimal, PricesError> {
        let rate = self.get_rate(src_commodity_name, dst_commodity_name, date)?;
        Ok(amount * rate)
    }

    pub fn get_rate(
        &self,
        src_commodity_name: &str,
        dst_commodity_name: &str,
        date: NaiveDate,
    ) -> Result<Decimal, PricesError> {
        let commodities_pair = CommoditiesPair::new(src_commodity_name, dst_commodity_name);

        self.get_rates_table(&commodities_pair)?.get_rate(date)
    }

    fn get_rates_table(
        &self,
        commodities_pair: &CommoditiesPair,
    ) -> Result<&RatesTable, PricesError> {
        self.rates
            .get(commodities_pair)
            .ok_or(PricesError::NoSuchCommoditiesPair(commodities_pair.clone()))
    }

    fn add_prices(&mut self, prices: &Vec<CommodityPrice>) {
        for price in prices {
            self.add_price(
                &price.commodity_name,
                &price.amount.commodity.name,
                price.amount.quantity,
                price.datetime.date(),
            );
            self.add_price(
                &price.amount.commodity.name,
                &price.commodity_name,
                Decimal::new(1, 0) / price.amount.quantity,
                price.datetime.date(),
            );
        }
    }

    fn add_price(
        &mut self,
        src_commodity_name: &str,
        dst_commodity_name: &str,
        rate: Decimal,
        date: NaiveDate,
    ) {
        let commodities_pair = CommoditiesPair::new(src_commodity_name, dst_commodity_name);
        self.rates
            .entry(commodities_pair)
            .or_insert_with(RatesTable::new)
            .table
            .entry(date)
            .and_modify(|r| *r = rate)
            .or_insert(rate);
    }
}

fn get_commodity_prices(ledger: &Ledger) -> Vec<CommodityPrice> {
    let mut result = Vec::new();
    for item in &ledger.items {
        if let LedgerItem::CommodityPrice(commodity_price) = item {
            result.push(commodity_price.clone());
        }
    }
    result
}

fn get_prices_from_transactions(ledger: &Ledger) -> Vec<CommodityPrice> {
    let mut result = Vec::new();
    for item in &ledger.items {
        if let LedgerItem::Transaction(transaction) = item {
            // TODO: handle empty amounts & balance verifications
            if transaction.postings.len() == 2
                && transaction.postings[0]
                .amount
                .clone()
                .unwrap()
                .commodity
                .name
                != transaction.postings[1]
                .amount
                .clone()
                .unwrap()
                .commodity
                .name
                && transaction.postings[0].amount.clone().unwrap().quantity != Decimal::new(0, 0)
                && transaction.postings[1].amount.clone().unwrap().quantity != Decimal::new(0, 0)
            {
                result.push(CommodityPrice {
                    datetime: transaction.date.and_hms(0, 0, 0),
                    commodity_name: (transaction.postings[0])
                        .amount
                        .clone()
                        .unwrap()
                        .commodity
                        .name,
                    amount: Amount {
                        quantity: -transaction.postings[1].amount.clone().unwrap().quantity
                            / transaction.postings[0].amount.clone().unwrap().quantity,
                        commodity: (transaction.postings[1]).amount.clone().unwrap().commodity,
                    },
                })
            }
        }
    }
    result
}
