use crate::amount::Amount;
use crate::funds::{not_frozen, FundingStates, Funds};
use serde::{de::Error, Deserializer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, PartialEq, Hash, Eq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
pub struct Tx(pub u32);
#[derive(Debug, PartialEq, Hash, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct Client(pub u16);

fn possible_null_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let mut s: &str = Deserialize::deserialize(deserializer)?;
    if s.is_empty() || s.to_lowercase() == "null" {
        s = "-1.0";
    }
    s.parse().map_err(D::Error::custom)
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Deserialize)]
pub struct RowRecord {
    r#type: TxType,
    client: Client,
    tx: Tx,
    #[serde(deserialize_with = "possible_null_f64")]
    amount: f64,
}

#[derive(Debug, PartialEq)]
pub struct TransactionRecord {
    pub r#type: TxType,
    pub amount: Option<Amount>,
    pub tx: Tx,
    pub client: Client,
}

#[derive(Debug, PartialEq)]
pub struct ProcessedRecord {
    pub r#type: TxType,
    pub amount: Amount,
    pub tx: Tx,
    pub client: Client,
}

impl From<RowRecord> for TransactionRecord {
    fn from(val: RowRecord) -> TransactionRecord {
        let amt = val.amount;
        let amount;
        if amt < 0 as f64 {
            amount = None
        } else {
            let amt_ = Amount::from_str(&amt.to_string()).unwrap();
            amount = Some(amt_)
        }
        TransactionRecord {
            client: val.client,
            tx: val.tx,
            amount: amount,
            r#type: val.r#type,
        }
    }
}

type ClientFunds = HashMap<Client, Funds>;

type TxRecords = HashMap<Tx, ProcessedRecord>;

fn valid_deposit(client: Option<&Funds>, record: &TransactionRecord) -> bool {
    match (record.r#type, client, record.amount) {
        (TxType::Deposit, Some(n), Some(_)) if not_frozen(n) => true,
        (TxType::Deposit, None, Some(_)) => true,
        _ => false,
    }
}

fn valid_withdrawal(client: Option<&Funds>, record: &TransactionRecord) -> bool {
    match (record.r#type, client, record.amount) {
        (TxType::Withdrawal, Some(n), Some(_)) if not_frozen(n) => true,
        _ => false,
    }
}

fn valid_dispute(
    client: Option<&Funds>,
    record: Option<&TransactionRecord>,
    previous_record: Option<&ProcessedRecord>,
) -> bool {
    if let (Some(fund), Some(tx_record), Some(previous)) = (client, record, previous_record) {
        match tx_record.r#type {
            TxType::Dispute if not_frozen(fund) && tx_record.tx == previous.tx => return true,
            _ => return false,
        }
    }
    return false;
}

fn valid_resolve(
    client: Option<&Funds>,
    record: Option<&TransactionRecord>,
    previous_record: Option<&ProcessedRecord>,
) -> bool {
    if let (Some(fund), Some(tx_record), Some(previous)) = (client, record, previous_record) {
        match (&fund.state, tx_record.r#type) {
            (FundingStates::Disputed, TxType::Resolve)
                if not_frozen(fund) && tx_record.tx == previous.tx =>
            {
                return true
            }
            _ => return false,
        }
    }
    return false;
}

fn valid_chargeback(
    client: Option<&Funds>,
    record: Option<&TransactionRecord>,
    previous_record: Option<&ProcessedRecord>,
) -> bool {
    if let (Some(fund), Some(tx_record), Some(previous)) = (client, record, previous_record) {
        match (&fund.state, tx_record.r#type) {
            (FundingStates::Disputed, TxType::Chargeback)
                if not_frozen(fund) && tx_record.tx == previous.tx =>
            {
                return true
            }
            _ => return false,
        }
    }
    return false;
}

fn transact(
    client_funds: &mut ClientFunds,
    records: &mut TxRecords,
    initial_record: &TransactionRecord,
) {
    let previous_record = records.get(&initial_record.tx);
    match (
        client_funds.get_mut(&initial_record.client),
        initial_record.r#type,
    ) {
        (Some(client), TxType::Deposit) if valid_deposit(Some(client), initial_record) => {
            client.deposit(initial_record.amount.unwrap())
        }
        (Some(client), TxType::Withdrawal) if valid_withdrawal(Some(client), initial_record) => {
            client.withdraw(initial_record.amount.unwrap())
        }
        (Some(client), TxType::Dispute)
            if valid_dispute(Some(client), Some(initial_record), previous_record) =>
        {
            client.dispute(previous_record.unwrap().amount)
        }
        (Some(client), TxType::Resolve)
            if valid_resolve(Some(client), Some(initial_record), previous_record) =>
        {
            client.resolve(previous_record.unwrap().amount)
        }
        (Some(client), TxType::Chargeback)
            if valid_chargeback(Some(client), Some(initial_record), previous_record) =>
        {
            client.chargeback(previous_record.unwrap().amount)
        }
        _ => (),
    };
}

#[cfg(test)]
mod tests {
    use super::{
        valid_chargeback, valid_deposit, valid_dispute, valid_resolve, Amount, Client, ClientFunds,
        FundingStates, Funds, ProcessedRecord, RowRecord, TransactionRecord, Tx, TxType,
    };
    use std::io::BufReader;

    #[test]
    fn it_serializes() {
        let csvfile = "type,client,tx,amount\ndeposit,1,1,1\ndeposit,2,2,2.0\ndeposit,1,3,2.0\ndispute,1,3,null\n";
        let buf_reader = BufReader::new(csvfile.as_bytes());
        let mut rdr = csv::Reader::from_reader(buf_reader);
        let mut rows: Vec<RowRecord> = Vec::new();
        for result in rdr.deserialize() {
            // Notice that we need to provide a type hint for automatic
            // deserialization.
            let record: RowRecord = result.unwrap();
            rows.push(record);
        }
        assert_eq!(4, rows.len());
        assert_eq!(
            rows[2],
            RowRecord {
                client: Client(1),
                tx: Tx(3),
                amount: 2.0,
                r#type: TxType::Deposit
            }
        );
        assert_eq!(
            rows[1],
            RowRecord {
                client: Client(2),
                tx: Tx(2),
                amount: 2.0,
                r#type: TxType::Deposit
            }
        );
        assert_eq!(
            rows[3],
            RowRecord {
                client: Client(1),
                tx: Tx(3),
                amount: -1.0,
                r#type: TxType::Dispute
            }
        )
    }
    #[test]
    fn from_str_transactionrecord() {
        let record = RowRecord {
            client: Client(1),
            tx: Tx(3),
            amount: -1.0,
            r#type: TxType::Dispute,
        };
        assert_eq!(
            TransactionRecord {
                client: Client(1),
                tx: Tx(3),
                amount: None,
                r#type: TxType::Dispute
            },
            TransactionRecord::from(record)
        );

        let other_record = RowRecord {
            client: Client(1),
            tx: Tx(3),
            amount: 0.1,
            r#type: TxType::Dispute,
        };
        assert_eq!(
            TransactionRecord {
                client: Client(1),
                tx: Tx(3),
                amount: Some(Amount::new(1000)),
                r#type: TxType::Dispute
            },
            TransactionRecord::from(other_record)
        )
    }
    #[test]
    fn test_valid_deposit() {
        let client_funds = ClientFunds::new();
        let record = TransactionRecord {
            client: Client(1),
            tx: Tx(3),
            amount: Some(Amount::new(1000)),
            r#type: TxType::Dispute,
        };
        assert!(!valid_deposit(client_funds.get(&Client(1)), &record));
        let deposit = TransactionRecord {
            client: Client(1),
            tx: Tx(3),
            amount: Some(Amount::new(1000)),
            r#type: TxType::Deposit,
        };
        assert!(valid_deposit(client_funds.get(&Client(1)), &deposit));
        let mut fund = Funds {
            state: FundingStates::Valid,
            available: Amount::new(1000),
            held: Amount::new(0),
            client: Client(5),
        };
        assert!(valid_deposit(Some(&fund), &deposit));
        fund.state = FundingStates::Frozen;
        assert!(!valid_deposit(Some(&fund), &deposit));
        fund.state = FundingStates::Disputed;
        assert!(valid_deposit(Some(&fund), &deposit));
    }
    #[test]
    fn test_valid_dispute() {
        let record = TransactionRecord {
            client: Client(1),
            tx: Tx(3),
            amount: None,
            r#type: TxType::Dispute,
        };
        let mut fund = Funds {
            state: FundingStates::Valid,
            available: Amount::new(1000),
            held: Amount::new(0),
            client: Client(1),
        };
        let prev = ProcessedRecord {
            client: Client(1),
            tx: Tx(3),
            amount: Amount(5),
            r#type: TxType::Dispute,
        };
        assert!(valid_dispute(Some(&fund), Some(&record), Some(&prev)));
        assert!(!valid_dispute(Some(&fund), None, Some(&prev)));
        assert!(!valid_dispute(Some(&fund), Some(&record), None));

        fund.state = FundingStates::Frozen;
        assert!(!valid_dispute(Some(&fund), Some(&record), Some(&prev)));
        assert!(!valid_dispute(Some(&fund), None, Some(&prev)));
        assert!(!valid_dispute(Some(&fund), None, None));

        fund.state = FundingStates::Disputed;
        assert!(valid_dispute(Some(&fund), Some(&record), Some(&prev)));
        assert!(!valid_dispute(Some(&fund), None, Some(&prev)));
        assert!(!valid_dispute(Some(&fund), Some(&record), None));

        assert!(!valid_dispute(None, Some(&record), Some(&prev)));
        assert!(!valid_dispute(None, None, None));
    }
    #[test]
    fn test_valid_resolve() {
        let mut record = TransactionRecord {
            client: Client(1),
            tx: Tx(5),
            amount: None,
            r#type: TxType::Resolve,
        };
        let prev = ProcessedRecord {
            client: Client(1),
            tx: Tx(5),
            amount: Amount(5),
            r#type: TxType::Resolve,
        };
        let mut fund = Funds {
            state: FundingStates::Disputed,
            available: Amount::new(1000),
            held: Amount::new(20),
            client: Client(1),
        };
        assert!(valid_resolve(Some(&fund), Some(&record), Some(&prev)));
        assert!(!valid_resolve(None, Some(&record), Some(&prev)));
        record.r#type = TxType::Dispute;
        assert!(!valid_resolve(None, Some(&record), Some(&prev)));

        fund.state = FundingStates::Valid;

        assert!(!valid_resolve(Some(&fund), Some(&record), None));
        assert!(!valid_resolve(None, None, None));
    }
    #[test]
    fn test_valid_chargeback() {
        let record = TransactionRecord {
            client: Client(1),
            tx: Tx(3),
            amount: None,
            r#type: TxType::Chargeback,
        };
        let mut fund = Funds {
            state: FundingStates::Disputed,
            available: Amount::new(1000),
            held: Amount::new(1000),
            client: Client(1),
        };
        let prev = ProcessedRecord {
            client: Client(1),
            tx: Tx(3),
            amount: Amount(5),
            r#type: TxType::Dispute,
        };
        assert!(valid_chargeback(Some(&fund), Some(&record), Some(&prev)));
        assert!(!valid_chargeback(None, None, None));
        assert!(!valid_chargeback(None, Some(&record), Some(&prev)));
        fund.state = FundingStates::Valid;
        assert!(!valid_chargeback(Some(&fund), Some(&record), Some(&prev)));
        fund.state = FundingStates::Frozen;
        assert!(!valid_chargeback(Some(&fund), Some(&record), None));
    }
}
