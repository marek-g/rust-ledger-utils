# ledger-utils

[![Crates.io Version](https://img.shields.io/crates/v/ledger-utils.svg)](https://crates.io/crates/ledger-utils)
[![Docs.rs Version](https://docs.rs/ledger-utils/badge.svg)](https://docs.rs/ledger-utils)
[![License Unlicense](https://img.shields.io/crates/l/ledger-utils.svg)](http://unlicense.org/UNLICENSE)

[Ledger-cli](https://www.ledger-cli.org/) file processing Rust library, useful for calculating balances, creating reports etc.

```rust
use anyhow::{bail, Result};
use ledger_utils::{balance::Balance, Ledger};

fn main() -> Result<()> {
    let ledger: Ledger = fs::read_to_string("finances.ledger")?.parse()?;

    if ledger.transactions.is_empty() {
        bail!("no transactions found");
    }

    let balance: Balance = (&ledger).into();

    let mut assets: Vec<_> = balance
        .account_balances
        .iter()
        .filter(|(name, _)| name.starts_with("Assets:"))
        .collect();
    assets.sort_by_key(|&(name, _)| name);
    for (name, balance) in assets {
        println!("{}: {}", name, balance);
    }

    Ok(())
}
```
