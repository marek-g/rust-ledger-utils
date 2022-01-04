use crate::account_balance::AccountBalance;
use ledger_parser::*;
use std::collections::HashMap;
use std::ops::AddAssign;
use std::ops::SubAssign;

/// Balance of one or more accounts.
///
/// Maps account names to their balances.
#[derive(Debug, Clone)]
pub struct Balance {
    pub account_balances: HashMap<String, AccountBalance>,
}

impl Balance {
    pub fn new() -> Balance {
        Balance {
            account_balances: HashMap::new(),
        }
    }

    pub fn update_with_transaction(&mut self, transaction: &Transaction) {
        for posting in &transaction.postings {
            let account_balance = self
                .account_balances
                .entry(posting.account.clone())
                .or_insert_with(AccountBalance::new);

            // TODO: handle empty amounts & balance verifications
            account_balance
                .amounts
                .entry(posting.amount.clone().unwrap().commodity.name)
                .and_modify(|a| a.quantity += posting.amount.clone().unwrap().quantity)
                .or_insert_with(|| posting.amount.clone().unwrap());
        }
    }

    pub fn get_account_balance(&self, account_prefixes: &[&str]) -> AccountBalance {
        let mut balance = AccountBalance::new();
        for (account_name, account_balance) in &self.account_balances {
            for account_prefix in account_prefixes {
                if account_name.starts_with(account_prefix) {
                    balance += account_balance;
                    break;
                }
            }
        }

        balance
    }

    fn remove_empties(&mut self) {
        let empties: Vec<String> = self
            .account_balances
            .iter()
            .filter(|&(_, account_balance)| account_balance.amounts.is_empty())
            .map(|(k, _)| k.clone())
            .collect();
        for empty in empties {
            self.account_balances.remove(&empty);
        }
    }
}

impl<'a> From<&'a Ledger> for Balance {
    fn from(ledger: &'a Ledger) -> Self {
        let mut balance = Balance::new();

        for item in &ledger.items {
            if let LedgerItem::Transaction(transaction) = item {
                balance.update_with_transaction(transaction);
            }
        }

        balance
    }
}

impl<'a> From<&'a Transaction> for Balance {
    fn from(transaction: &'a Transaction) -> Self {
        let mut balance = Balance::new();
        balance.update_with_transaction(transaction);
        balance
    }
}

impl<'a> AddAssign<&'a Balance> for Balance {
    fn add_assign(&mut self, other: &'a Balance) {
        for (account_name, account_balance) in &other.account_balances {
            self.account_balances
                .entry(account_name.clone())
                .and_modify(|b| *b += account_balance)
                .or_insert_with(|| account_balance.clone());
        }
        self.remove_empties();
    }
}

impl<'a> SubAssign<&'a Balance> for Balance {
    fn sub_assign(&mut self, other: &'a Balance) {
        for (account_name, account_balance) in &other.account_balances {
            self.account_balances
                .entry(account_name.clone())
                .and_modify(|b| *b -= account_balance)
                .or_insert_with(|| account_balance.clone());
        }
        self.remove_empties();
    }
}
