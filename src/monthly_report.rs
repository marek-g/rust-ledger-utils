use crate::balance::Balance;
use crate::Ledger;
use chrono::Datelike;

#[derive(Debug, Clone)]
pub struct MonthlyBalance {
    pub year: i32,
    pub month: u32,
    pub monthly_change: Balance,
    pub total: Balance,
}

impl MonthlyBalance {
    pub fn new(year: i32, month: u32) -> MonthlyBalance {
        MonthlyBalance {
            year,
            month,
            monthly_change: Balance::new(),
            total: Balance::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonthlyReport {
    pub monthly_balances: Vec<MonthlyBalance>,
}

impl Default for MonthlyReport {
    fn default() -> Self {
        Self::new()
    }
}

impl MonthlyReport {
    pub fn new() -> MonthlyReport {
        MonthlyReport {
            monthly_balances: Vec::new(),
        }
    }
}

impl<'a> From<&'a Ledger> for MonthlyReport {
    fn from(ledger: &'a Ledger) -> Self {
        let mut report = MonthlyReport::new();

        let mut current_year = 0;
        let mut current_month = 0;
        let mut current_monthly_balance: Option<MonthlyBalance> = None;
        let mut monthly_balance = Balance::new();
        let mut total_balance = Balance::new();

        for transaction in &ledger.transactions {
            if transaction.date.year() != current_year || transaction.date.month() != current_month
            {
                // begin new month

                if let Some(mut b) = current_monthly_balance.take() {
                    b.monthly_change = monthly_balance.clone();
                    b.total = total_balance.clone();
                    report.monthly_balances.push(b);
                }

                current_year = transaction.date.year();
                current_month = transaction.date.month();
                monthly_balance = Balance::new();

                current_monthly_balance = Some(MonthlyBalance::new(current_year, current_month));
            }

            monthly_balance.update_with_transaction(transaction);
            total_balance.update_with_transaction(transaction);
        }

        if let Some(mut b) = current_monthly_balance.take() {
            b.monthly_change = monthly_balance.clone();
            b.total = total_balance.clone();
            report.monthly_balances.push(b);
        }

        report
    }
}
