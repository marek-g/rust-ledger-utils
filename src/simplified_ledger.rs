use crate::*;
use chrono::NaiveDate;
use ledger_parser::{LedgerItem, Serializer, SerializerSettings, Tag, TagValue};
use std::str::FromStr;
use std::{fmt, io};

///
/// Main document. Contains transactions and/or commodity prices.
///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ledger {
    pub commodity_prices: Vec<ledger_parser::CommodityPrice>,
    pub transactions: Vec<Transaction>,
}

impl fmt::Display for Ledger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_string_pretty(&SerializerSettings::default())
        )?;
        Ok(())
    }
}

impl Serializer for Ledger {
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

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ParseError(ledger_parser::ParseError),
    IncompleteTransaction(Box<ledger_parser::Posting>),
    UnbalancedTransaction(Box<ledger_parser::Transaction>),
    BalanceAssertionFailed(Box<ledger_parser::Transaction>),
    ZeroBalanceAssertionFailed(Box<ledger_parser::Transaction>),
    UnbalancedVirtualWithNoAmount(Box<ledger_parser::Transaction>),
    ZeroBalanceMultipleCurrencies(Box<ledger_parser::Transaction>),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ParseError(p) => {
                write!(f, "Parse error:\n{}", p)
            }
            Error::IncompleteTransaction(p) => {
                write!(f, "Incomplete transaction:\n{}", p)
            }
            Error::UnbalancedTransaction(t) => {
                write!(f, "Unbalanced transaction:\n{}", t)
            }
            Error::BalanceAssertionFailed(t) => {
                write!(f, "Balance assertion failed:\n{}", t)
            }
            Error::ZeroBalanceAssertionFailed(t) => {
                write!(f, "Zero balance assertion failed:\n{}", t)
            }
            Error::UnbalancedVirtualWithNoAmount(t) => {
                write!(f, "Unbalanced virtual posting with no amount:\n{}", t)
            }
            Error::ZeroBalanceMultipleCurrencies(t) => {
                write!(f, "Zero balance with multiple currencies:\n{}", t)
            }
        }
    }
}

impl From<ledger_parser::ParseError> for Error {
    fn from(e: ledger_parser::ParseError) -> Self {
        Error::ParseError(e)
    }
}

impl FromStr for Ledger {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        input.parse::<ledger_parser::Ledger>()?.try_into()
    }
}

impl TryFrom<ledger_parser::Ledger> for Ledger {
    type Error = Error;

    /// Fails if any transactions are unbalanced, any balance assertions fail, or if an unbalanced
    /// virtual posting (account name in `()`) has no amount.
    ///
    /// "Balance assertions" are postings with both amount and balance provided. The calculated
    /// amount using the balance must match the given amount.
    fn try_from(ledger: ledger_parser::Ledger) -> Result<Self, Self::Error> {
        let mut transactions = Vec::<ledger_parser::Transaction>::new();
        let mut commodity_prices = Vec::<ledger_parser::CommodityPrice>::new();

        let mut current_comment: Option<String> = None;

        for item in ledger.items {
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

        calculate_amounts::calculate_amounts_from_balances(
            &mut transactions,
            &mut commodity_prices,
        )?;

        Ok(Ledger {
            transactions: transactions
                .into_iter()
                .map(Transaction::try_from)
                .collect::<Result<_, _>>()?,
            commodity_prices,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Transaction {
    pub comment: Option<String>,
    pub date: NaiveDate,
    pub effective_date: NaiveDate,
    pub status: Option<TransactionStatus>,
    pub code: Option<String>,
    pub description: String,
    pub postings: Vec<Posting>,
}

impl TryFrom<ledger_parser::Transaction> for Transaction {
    type Error = Error;

    /// Fails if any transactions are unbalanced, or if an unbalanced virtual posting
    /// (account name in `()`) has no amount.
    ///
    /// Ignores `balance`s. Fails if they are necessary to fill in any omitted `amount`s.
    fn try_from(mut transaction: ledger_parser::Transaction) -> Result<Self, Self::Error> {
        calculate_amounts::calculate_omitted_amounts(&mut transaction)?;

        Ok(Transaction {
            comment: transaction.comment,
            date: transaction.date,
            effective_date: transaction.effective_date.unwrap_or(transaction.date),
            status: transaction.status,
            code: transaction.code,
            description: transaction.description,
            postings: transaction
                .postings
                .into_iter()
                .map(OptionalDatePosting::try_from)
                .map(|res| {
                    res.map(|posting| {
                        posting.fill_dates(transaction.date, transaction.effective_date)
                    })
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

impl Serializer for Transaction {
    fn write<W>(&self, writer: &mut W, settings: &SerializerSettings) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        write!(writer, "{}", self.date.format("%Y-%m-%d"))?;

        if self.effective_date != self.date {
            write!(writer, "={}", self.effective_date.format("%Y-%m-%d"))?;
        }

        if let Some(ref status) = self.status {
            write!(writer, " ")?;
            status.write(writer, settings)?;
        }

        if let Some(ref code) = self.code {
            write!(writer, " ({})", code)?;
        }

        if !self.description.is_empty() {
            write!(writer, " {}", self.description)?;
        }

        if let Some(ref comment) = self.comment {
            for comment in comment.split('\n') {
                write!(writer, "{}{}; {}", settings.eol, settings.indent, comment)?;
            }
        }

        for posting in &self.postings {
            write!(writer, "{}{}", settings.eol, settings.indent)?;
            posting.elide_dates(self).write(writer, settings)?;
        }

        Ok(())
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_string_pretty(&SerializerSettings::default())
        )?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OptionalDatePosting {
    pub date: Option<NaiveDate>,
    pub effective_date: Option<NaiveDate>,
    pub account: String,
    pub reality: Reality,
    pub amount: Amount,
    pub status: Option<TransactionStatus>,
    pub comment: Option<String>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Posting {
    pub date: NaiveDate,
    pub effective_date: NaiveDate,
    pub account: String,
    pub reality: Reality,
    pub amount: Amount,
    pub status: Option<TransactionStatus>,
    pub comment: Option<String>,
    pub tags: Vec<Tag>,
}

impl OptionalDatePosting {
    pub fn fill_dates(self, txn_date: NaiveDate, txn_effective_date: Option<NaiveDate>) -> Posting {
        Posting {
            date: self.date.unwrap_or(txn_date),
            effective_date: self
                .effective_date
                .or(self.date)
                .or(txn_effective_date)
                .unwrap_or(txn_date),
            account: self.account,
            reality: self.reality,
            amount: self.amount,
            status: self.status,
            comment: self.comment,
            tags: self.tags,
        }
    }
}

impl Posting {
    pub fn elide_dates(&self, txn: &Transaction) -> OptionalDatePosting {
        let date = if self.date != txn.date {
            Some(self.date)
        } else {
            None
        };

        let effective_date = if self.effective_date != date.unwrap_or(txn.effective_date) {
            Some(self.effective_date)
        } else {
            None
        };

        OptionalDatePosting {
            date,
            effective_date,
            account: self.account.clone(),
            reality: self.reality,
            amount: self.amount.clone(),
            status: self.status,
            comment: self.comment.clone(),
            tags: self.tags.clone(),
        }
    }
}

impl TryFrom<ledger_parser::Posting> for OptionalDatePosting {
    type Error = Error;

    /// Fails unless all `amount`s are `Some`. Ignores `balance`s.
    fn try_from(posting: ledger_parser::Posting) -> Result<Self, Self::Error> {
        if let Some(ledger_parser::PostingAmount { amount, .. }) = posting.amount {
            Ok(Self {
                date: posting.metadata.date,
                effective_date: posting.metadata.effective_date,
                account: posting.account,
                reality: posting.reality,
                status: posting.status,
                comment: posting.comment,
                amount,
                tags: posting.metadata.tags,
            })
        } else {
            Err(Error::IncompleteTransaction(posting.into()))
        }
    }
}

impl Serializer for OptionalDatePosting {
    fn write<W>(&self, writer: &mut W, settings: &SerializerSettings) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        if let Some(ref status) = self.status {
            status.write(writer, settings)?;
            write!(writer, " ")?;
        }

        match self.reality {
            Reality::Real => write!(writer, "{}", self.account)?,
            Reality::BalancedVirtual => write!(writer, "[{}]", self.account)?,
            Reality::UnbalancedVirtual => write!(writer, "({})", self.account)?,
        }

        write!(writer, "  ")?;
        self.amount.write(writer, settings)?;

        let mut first = true;

        if let Some(ref comment) = self.comment {
            for comment in comment.split('\n') {
                if first {
                    first = false;
                    write!(writer, "  ")?;
                } else {
                    write!(writer, "{}{}", settings.eol, settings.indent)?;
                }
                write!(writer, "; {}", comment)?;
            }
        }

        if self.date.is_some() || self.effective_date.is_some() {
            if first {
                first = false;
                write!(writer, "  ")?;
            } else {
                write!(writer, "{}{}", settings.eol, settings.indent)?;
            }
            write!(writer, "; [")?;
            if let Some(d) = self.date {
                write!(writer, "{d}")?;
            }
            if let Some(d) = self.effective_date {
                write!(writer, "={d}")?;
            }
            write!(writer, "]")?;
        }

        let (tags, tags_with_values): (Vec<_>, Vec<_>) =
            self.tags.iter().partition(|t| t.value.is_none());

        if !tags.is_empty() {
            if first {
                first = false;
                write!(writer, "  ")?;
            } else {
                write!(writer, "{}{}", settings.eol, settings.indent)?;
            }
            write!(writer, "; :")?;
            for tag in tags {
                write!(writer, "{}:", tag.name)?;
            }
        }

        for tag in tags_with_values {
            if first {
                first = false;
                write!(writer, "  ")?;
            } else {
                write!(writer, "{}{}", settings.eol, settings.indent)?;
            }
            match &tag.value {
                Some(TagValue::String(s)) => write!(writer, "; {}: {s}", tag.name)?,
                Some(other_type) => write!(writer, "; {}:: {other_type}", tag.name)?,
                None => unreachable!(),
            }
        }

        Ok(())
    }
}

impl fmt::Display for OptionalDatePosting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_string_pretty(&SerializerSettings::default())
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use ledger_parser::{Amount, Commodity, CommodityPosition, CommodityPrice, Reality};
    use rust_decimal::Decimal;

    #[test]
    fn test_handle_commodity_exchange() {
        let ledger = ledger_parser::parse(
            r#"
2022-02-19 Exchange
  DollarAccount   $1.00
  PLNAccount  -4.00 PLN
"#,
        )
        .unwrap();
        let simplified_ledger: Result<Ledger, _> = ledger.try_into();
        assert!(simplified_ledger.is_ok());
        assert_eq!(simplified_ledger.unwrap().commodity_prices.len(), 1);
    }

    #[test]
    fn test_handle_commodity_exchange2() {
        let ledger = ledger_parser::parse(
            r#"
2020-02-01 Buy ADA
  assets:cc:ada          2000 ADA @ $0.02
  assets:bank:checking                   $-40
"#,
        )
        .unwrap();
        let simplified_ledger: Result<Ledger, _> = ledger.try_into();
        assert!(simplified_ledger.is_ok());
        assert_eq!(simplified_ledger.unwrap().commodity_prices.len(), 1);
    }

    #[test]
    fn display_ledger() {
        let actual = format!(
            "{}",
            Ledger {
                transactions: vec![
                    Transaction {
                        comment: Some("Comment Line 1\nComment Line 2".to_string()),
                        date: NaiveDate::from_ymd_opt(2018, 10, 1).unwrap(),
                        effective_date: NaiveDate::from_ymd_opt(2018, 10, 14).unwrap(),
                        status: Some(TransactionStatus::Pending),
                        code: Some("123".to_string()),
                        description: "Marek Ogarek".to_string(),
                        postings: vec![
                            Posting {
                                date: NaiveDate::from_ymd_opt(2018, 10, 1).unwrap(),
                                effective_date: NaiveDate::from_ymd_opt(2018, 10, 14).unwrap(),
                                account: "TEST:ABC 123".to_string(),
                                reality: Reality::Real,
                                amount: Amount {
                                    quantity: Decimal::new(120, 2),
                                    commodity: Commodity {
                                        name: "$".to_string(),
                                        position: CommodityPosition::Left
                                    }
                                },
                                status: None,
                                comment: Some("dd".to_string()),
                                tags: vec![],
                            },
                            Posting {
                                date: NaiveDate::from_ymd_opt(2018, 10, 1).unwrap(),
                                effective_date: NaiveDate::from_ymd_opt(2018, 10, 14).unwrap(),
                                account: "TEST:ABC 123".to_string(),
                                reality: Reality::Real,
                                amount: Amount {
                                    quantity: Decimal::new(120, 2),
                                    commodity: Commodity {
                                        name: "$".to_string(),
                                        position: CommodityPosition::Left
                                    }
                                },
                                status: None,
                                comment: None,
                                tags: vec![
                                    Tag {
                                        name: "Tag1".to_string(),
                                        value: None
                                    },
                                    Tag {
                                        name: "Tag2".to_string(),
                                        value: None
                                    }
                                ],
                            }
                        ]
                    },
                    Transaction {
                        comment: None,
                        date: NaiveDate::from_ymd_opt(2018, 10, 1).unwrap(),
                        effective_date: NaiveDate::from_ymd_opt(2018, 10, 1).unwrap(),
                        status: None,
                        code: None,
                        description: "Marek Ogarek".to_string(),
                        postings: vec![
                            Posting {
                                date: NaiveDate::from_ymd_opt(2018, 10, 1).unwrap(),
                                effective_date: NaiveDate::from_ymd_opt(2018, 10, 1).unwrap(),
                                account: "TEST:ABC 123".to_string(),
                                reality: Reality::Real,
                                amount: Amount {
                                    quantity: Decimal::new(120, 2),
                                    commodity: Commodity {
                                        name: "$".to_string(),
                                        position: CommodityPosition::Left
                                    }
                                },
                                status: None,
                                comment: None,
                                tags: vec![Tag {
                                    name: "DateTag".to_string(),
                                    value: Some(TagValue::Date(
                                        NaiveDate::from_ymd_opt(2017, 12, 31).unwrap()
                                    ))
                                }],
                            },
                            Posting {
                                date: NaiveDate::from_ymd_opt(2017, 12, 30).unwrap(),
                                effective_date: NaiveDate::from_ymd_opt(2017, 12, 30).unwrap(),
                                account: "TEST:ABC 123".to_string(),
                                reality: Reality::Real,
                                amount: Amount {
                                    quantity: Decimal::new(120, 2),
                                    commodity: Commodity {
                                        name: "$".to_string(),
                                        position: CommodityPosition::Left
                                    }
                                },
                                status: None,
                                comment: None,
                                tags: vec![],
                            }
                        ]
                    }
                ],
                commodity_prices: vec![CommodityPrice {
                    datetime: NaiveDate::from_ymd_opt(2017, 11, 12)
                        .unwrap()
                        .and_hms_opt(12, 00, 00)
                        .unwrap(),
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
  TEST:ABC 123  $1.20  ; dd
  TEST:ABC 123  $1.20  ; :Tag1:Tag2:

2018-10-01 Marek Ogarek
  TEST:ABC 123  $1.20  ; DateTag:: [2017-12-31]
  TEST:ABC 123  $1.20  ; [2017-12-30]
"#;
        assert_eq!(actual, expected);
    }
}
