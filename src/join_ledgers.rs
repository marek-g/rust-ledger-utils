use ledger_parser::Ledger;

pub fn join_ledgers(ledgers: Vec<Ledger>) -> Ledger {
    let mut ledger = Ledger {
        items: Vec::new(),
    };

    for mut src_ledger in ledgers {
        ledger.items.append(&mut src_ledger.items);
    }

    ledger
}
