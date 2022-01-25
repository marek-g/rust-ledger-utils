use crate::Ledger;

pub fn join_ledgers(ledgers: Vec<Ledger>) -> Ledger {
    let mut ledger = Ledger {
        commodity_prices: Vec::new(),
        transactions: Vec::new(),
    };

    for mut src_ledger in ledgers {
        ledger
            .commodity_prices
            .append(&mut src_ledger.commodity_prices);
        ledger.transactions.append(&mut src_ledger.transactions);
    }

    ledger.commodity_prices.sort_by_key(|price| price.datetime);
    ledger.transactions.sort_by_key(|txn| txn.date);

    ledger
}
