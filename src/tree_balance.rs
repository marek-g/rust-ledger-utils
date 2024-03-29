use crate::account_balance::AccountBalance;
use crate::balance::Balance;
use std::collections::HashMap;

/// Balance of one or more accounts.
/// Converted to a tree.
#[derive(Debug, Clone)]
pub struct TreeBalanceNode {
    pub balance: AccountBalance,
    pub children: HashMap<String, TreeBalanceNode>,
}

impl TreeBalanceNode {
    pub fn new() -> Self {
        TreeBalanceNode {
            balance: AccountBalance::new(),
            children: HashMap::new(),
        }
    }
}

impl Default for TreeBalanceNode {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Balance> for TreeBalanceNode {
    fn from(balance: Balance) -> Self {
        let mut root = TreeBalanceNode::new();

        for (account_name, account_balance) in balance.account_balances {
            let path = account_name.split(':');
            let mut node = &mut root;
            node.balance += &account_balance;

            for path_part in path {
                node = node.children.entry(path_part.to_string()).or_default();
                node.balance += &account_balance;
            }
        }

        root
    }
}
