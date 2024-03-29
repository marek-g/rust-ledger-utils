use crate::prices::{Prices, PricesError};
use crate::{Amount, Commodity, CommodityPosition, Ledger, Posting, Reality, Transaction};
use rust_decimal::RoundingStrategy;

/// Handle foreign currencies.
/// Generate additional postings for "currency trading accounts".
/// This is a method to properly keep track of currency gains and losses.
pub fn handle_foreign_currencies<F1, F2, F3>(
    ledger: &mut Ledger,
    is_asset_account: &F1,
    is_income_account: &F2,
    is_expense_account: &F3,
    main_commodity: &str,
    main_commodity_decimal_points: u32,
    prices: &Prices,
) -> Result<(), PricesError>
where
    F1: Fn(&str) -> bool,
    F2: Fn(&str) -> bool,
    F3: Fn(&str) -> bool,
{
    for transaction in &mut ledger.transactions {
        handle_foreign_asset_income(
            transaction,
            is_income_account,
            main_commodity,
            main_commodity_decimal_points,
            prices,
        )?;
        handle_asset_exchange(transaction, is_asset_account);
        handle_foreign_asset_expenses(
            transaction,
            is_expense_account,
            main_commodity,
            main_commodity_decimal_points,
            prices,
        )?;
    }
    Ok(())
}

/// Every time there is an income in foreign currency,
/// change it to main_currency so its value is frozen in time
/// and update currency trading account
/// so that the value of trading account equals currency gains and losses in time.
fn handle_foreign_asset_income<F>(
    transaction: &mut Transaction,
    is_income_account: &F,
    main_commodity: &str,
    main_commodity_decimal_points: u32,
    prices: &Prices,
) -> Result<(), PricesError>
where
    F: Fn(&str) -> bool,
{
    let mut new_postings = Vec::new();

    // look for postings that spends foreign commodities
    for posting in transaction.postings.iter_mut() {
        if is_income_account(&posting.account) && posting.amount.commodity.name != main_commodity {
            let foreign_amount = posting.amount.clone();

            // convert amount to main commodity
            let mut amount_main_commodity = prices.convert(
                posting.amount.quantity,
                &posting.amount.commodity.name,
                main_commodity,
                transaction.date,
            )?;
            amount_main_commodity = amount_main_commodity.round_dp_with_strategy(
                main_commodity_decimal_points,
                RoundingStrategy::MidpointAwayFromZero,
            );

            // replace the value
            let mut main_currency_amount = Amount {
                quantity: amount_main_commodity,
                commodity: Commodity {
                    name: main_commodity.to_string(),
                    position: CommodityPosition::Right,
                },
            };
            posting.amount = main_currency_amount.clone();

            // add postings to trading account that will track currency gains and losses
            main_currency_amount.quantity = -main_currency_amount.quantity;
            new_postings.push(Posting {
                date: posting.date,
                effective_date: posting.effective_date,
                comment: Some("Auto-generated".to_string()),
                account: "Trading:Exchange".to_string(),
                reality: Reality::Real,
                status: None,
                amount: main_currency_amount,
                tags: vec![],
            });
            new_postings.push(Posting {
                date: posting.date,
                effective_date: posting.effective_date,
                comment: Some("Auto-generated".to_string()),
                account: "Trading:Exchange".to_string(),
                reality: Reality::Real,
                status: None,
                amount: foreign_amount,
                tags: vec![],
            });
        }
    }

    transaction.postings.append(&mut new_postings);

    Ok(())
}

/// Every time there is an exchange made between assets,
/// add entries to corresponding currency trading account
/// so that the value of trading account equals currency gains and losses in time.
fn handle_asset_exchange<F>(transaction: &mut Transaction, is_asset_account: &F)
where
    F: Fn(&str) -> bool,
{
    // is this a transaction between two asset accounts
    if transaction.postings.len() != 2 {
        return;
    }

    let posting1 = &transaction.postings[0];
    let posting2 = &transaction.postings[1];

    if !is_asset_account(&posting1.account) || !is_asset_account(&posting2.account) {
        return;
    }

    // is this a transaction between different commodities
    let commodity1_name = &posting1.amount.commodity.name;
    let commodity2_name = &posting2.amount.commodity.name;
    if commodity1_name == commodity2_name {
        return;
    }

    // add postings to trading account that will track currency gains and losses
    let mut amount1 = posting1.amount.clone();
    let mut amount2 = posting2.amount.clone();

    amount1.quantity = -amount1.quantity;
    amount2.quantity = -amount2.quantity;

    let new_posting1 = Posting {
        date: posting1.date,
        effective_date: posting1.effective_date,
        comment: Some("Auto-generated".to_string()),
        account: "Trading:Exchange".to_string(),
        reality: Reality::Real,
        status: posting1.status,
        amount: amount1,
        tags: posting1.tags.clone(),
    };
    let new_posting2 = Posting {
        date: posting2.date,
        effective_date: posting2.effective_date,
        comment: Some("Auto-generated".to_string()),
        account: "Trading:Exchange".to_string(),
        reality: Reality::Real,
        status: posting2.status,
        amount: amount2,
        tags: posting2.tags.clone(),
    };

    transaction.postings.push(new_posting1);
    transaction.postings.push(new_posting2);
}

/// Every time there is an expense in foreign currency,
/// change it to main_currency so its value is frozen in time
/// and update currency trading account
/// so that the value of trading account equals currency gains and losses in time.
fn handle_foreign_asset_expenses<F>(
    transaction: &mut Transaction,
    is_expense_account: &F,
    main_commodity: &str,
    main_commodity_decimal_points: u32,
    prices: &Prices,
) -> Result<(), PricesError>
where
    F: Fn(&str) -> bool,
{
    let mut new_postings = Vec::new();

    // look for postings that spends foreign commodities
    for posting in transaction.postings.iter_mut() {
        if is_expense_account(&posting.account) && posting.amount.commodity.name != main_commodity {
            let foreign_amount = posting.amount.clone();

            // convert amount to main commodity
            let mut amount_main_commodity = prices.convert(
                posting.amount.quantity,
                &posting.amount.commodity.name,
                main_commodity,
                transaction.date,
            )?;
            amount_main_commodity = amount_main_commodity.round_dp_with_strategy(
                main_commodity_decimal_points,
                RoundingStrategy::MidpointAwayFromZero,
            );

            // replace the value
            let mut main_currency_amount = Amount {
                quantity: amount_main_commodity,
                commodity: Commodity {
                    name: main_commodity.to_string(),
                    position: CommodityPosition::Right,
                },
            };
            posting.amount = main_currency_amount.clone();

            // add postings to trading account that will track currency gains and losses
            main_currency_amount.quantity = -main_currency_amount.quantity;
            new_postings.push(Posting {
                date: posting.date,
                effective_date: posting.effective_date,
                comment: Some("Auto-generated".to_string()),
                account: "Trading:Exchange".to_string(),
                reality: Reality::Real,
                status: posting.status,
                amount: main_currency_amount,
                tags: posting.tags.clone(),
            });
            new_postings.push(Posting {
                date: posting.date,
                effective_date: posting.effective_date,
                comment: Some("Auto-generated".to_string()),
                account: "Trading:Exchange".to_string(),
                reality: Reality::Real,
                status: posting.status,
                amount: foreign_amount,
                tags: posting.tags.clone(),
            });
        }
    }

    transaction.postings.append(&mut new_postings);

    Ok(())
}
