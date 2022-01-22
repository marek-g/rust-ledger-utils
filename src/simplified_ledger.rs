use ledger_parser::*;
use std::{fmt, io};

///
/// Main document. Contains transactions and/or commodity prices.
///
/// TODO: this was previously the result type from ledger-parser crate.
/// Consider if this type is needed anymore.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimplifiedLedger {
    pub commodity_prices: Vec<CommodityPrice>,
    // TODO: simplify transactions to auto calculate omitted amounts
    pub transactions: Vec<Transaction>,
}

impl fmt::Display for SimplifiedLedger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_string_pretty(&SerializerSettings::default())
        )?;
        Ok(())
    }
}

impl Serializer for SimplifiedLedger {
    fn write<W>(&self, writer: &mut W, settings: &SerializerSettings) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        let mut first = true;

        for commodity_price in &self.commodity_prices {
            first = false;
            commodity_price.write(writer, settings)?;
            writeln!(writer)?;
        }

        for transaction in &self.transactions {
            if !first {
                writeln!(writer)?;
            }

            first = false;
            transaction.write(writer, settings)?;
            writeln!(writer)?;
        }

        Ok(())
    }
}

impl From<Ledger> for SimplifiedLedger {
    fn from(value: Ledger) -> Self {
        let mut transactions = Vec::<Transaction>::new();
        let mut commodity_prices = Vec::<CommodityPrice>::new();

        let mut current_comment: Option<String> = None;

        for item in value.items {
            match item {
                LedgerItem::EmptyLine => {
                    current_comment = None;
                }
                LedgerItem::LineComment(comment) => {
                    if let Some(ref mut c) = current_comment {
                        c.push('\n');
                        c.push_str(&comment);
                    } else {
                        current_comment = Some(comment);
                    }
                }
                LedgerItem::Transaction(mut transaction) => {
                    if let Some(current_comment) = current_comment {
                        let mut full_comment = current_comment;
                        if let Some(ref transaction_comment) = transaction.comment {
                            full_comment.push('\n');
                            full_comment.push_str(transaction_comment);
                        }
                        transaction.comment = Some(full_comment);
                    }
                    current_comment = None;

                    transactions.push(transaction);
                }
                LedgerItem::CommodityPrice(commodity_price) => {
                    current_comment = None;
                    commodity_prices.push(commodity_price);
                }
                _ => {}
            }
        }

        SimplifiedLedger {
            transactions,
            commodity_prices,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    #[test]
    fn display_ledger() {
        let actual = format!(
            "{}",
            SimplifiedLedger {
                transactions: vec![
                    Transaction {
                        comment: Some("Comment Line 1\nComment Line 2".to_string()),
                        date: NaiveDate::from_ymd(2018, 10, 1),
                        effective_date: Some(NaiveDate::from_ymd(2018, 10, 14)),
                        status: Some(TransactionStatus::Pending),
                        code: Some("123".to_string()),
                        description: "Marek Ogarek".to_string(),
                        postings: vec![
                            Posting {
                                account: "TEST:ABC 123".to_string(),
                                reality: Reality::Real,
                                amount: Some(PostingAmount {
                                    amount: Amount {
                                        quantity: Decimal::new(120, 2),
                                        commodity: Commodity {
                                            name: "$".to_string(),
                                            position: CommodityPosition::Left
                                        }
                                    },
                                    lot_price: None,
                                    price: None
                                }),
                                balance: None,
                                status: None,
                                comment: Some("dd".to_string())
                            },
                            Posting {
                                account: "TEST:ABC 123".to_string(),
                                reality: Reality::Real,
                                amount: Some(PostingAmount {
                                    amount: Amount {
                                        quantity: Decimal::new(120, 2),
                                        commodity: Commodity {
                                            name: "$".to_string(),
                                            position: CommodityPosition::Left
                                        }
                                    },
                                    lot_price: None,
                                    price: None
                                }),
                                balance: None,
                                status: None,
                                comment: None
                            }
                        ]
                    },
                    Transaction {
                        comment: None,
                        date: NaiveDate::from_ymd(2018, 10, 1),
                        effective_date: Some(NaiveDate::from_ymd(2018, 10, 14)),
                        status: Some(TransactionStatus::Pending),
                        code: Some("123".to_string()),
                        description: "Marek Ogarek".to_string(),
                        postings: vec![
                            Posting {
                                account: "TEST:ABC 123".to_string(),
                                reality: Reality::Real,
                                amount: Some(PostingAmount {
                                    amount: Amount {
                                        quantity: Decimal::new(120, 2),
                                        commodity: Commodity {
                                            name: "$".to_string(),
                                            position: CommodityPosition::Left
                                        }
                                    },
                                    lot_price: None,
                                    price: None
                                }),
                                balance: None,
                                status: None,
                                comment: None
                            },
                            Posting {
                                account: "TEST:ABC 123".to_string(),
                                reality: Reality::Real,
                                amount: Some(PostingAmount {
                                    amount: Amount {
                                        quantity: Decimal::new(120, 2),
                                        commodity: Commodity {
                                            name: "$".to_string(),
                                            position: CommodityPosition::Left
                                        }
                                    },
                                    lot_price: None,
                                    price: None
                                }),
                                balance: None,
                                status: None,
                                comment: None
                            }
                        ]
                    }
                ],
                commodity_prices: vec![CommodityPrice {
                    datetime: NaiveDate::from_ymd(2017, 11, 12).and_hms(12, 00, 00),
                    commodity_name: "mBH".to_string(),
                    amount: Amount {
                        quantity: Decimal::new(500, 2),
                        commodity: Commodity {
                            name: "PLN".to_string(),
                            position: CommodityPosition::Right
                        }
                    }
                }]
            }
        );
        let expected = r#"P 2017-11-12 12:00:00 mBH 5.00 PLN

2018-10-01=2018-10-14 ! (123) Marek Ogarek
  ; Comment Line 1
  ; Comment Line 2
  TEST:ABC 123  $1.20
  ; dd
  TEST:ABC 123  $1.20

2018-10-01=2018-10-14 ! (123) Marek Ogarek
  TEST:ABC 123  $1.20
  TEST:ABC 123  $1.20
"#;
        assert_eq!(actual, expected);
    }
}
