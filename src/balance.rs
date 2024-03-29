use crate::account_balance::AccountBalance;
use crate::{Amount, Ledger, Transaction};
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

impl Default for Balance {
    fn default() -> Self {
        Self::new()
    }
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
                .or_default();

            account_balance
                .amounts
                .entry(posting.amount.commodity.name.clone())
                .and_modify(|a| a.quantity += posting.amount.quantity)
                .or_insert_with(|| posting.amount.clone());
        }
        self.remove_empties();
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

    pub fn add_amount(&mut self, account: &str, amount: &Amount) {
        let account_balance = self.account_balances.entry(account.to_owned()).or_default();
        *account_balance += amount;
    }

    fn remove_empties(&mut self) {
        let empties: Vec<String> = self
            .account_balances
            .iter()
            .filter(|&(_, account_balance)| account_balance.is_zero())
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

        for transaction in &ledger.transactions {
            balance.update_with_transaction(transaction);
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
