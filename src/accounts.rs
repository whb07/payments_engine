use crate::amount::{Amount, RecordFloatAmount};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub struct Tx(pub u32);

#[derive(Debug, Deserialize, Serialize, PartialEq, Hash, Eq, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Copy, Clone)]
pub struct RowRecord {
    r#type: TxType,
    client: Client,
    tx: Tx,
    amount: RecordFloatAmount,
}

#[derive(Debug, PartialEq)]
pub struct TransactionRecord {
    pub r#type: TxType,
    pub amount: Amount,
    pub tx: Tx,
    client: Client,
}

impl From<RowRecord> for TransactionRecord {
    fn from(val: RowRecord) -> TransactionRecord {
        TransactionRecord {
            r#type: val.r#type,
            tx: val.tx,
            amount: Amount::from(val.amount),
            client: val.client,
        }
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
pub struct Client(u16);

pub mod records {
    use super::{Client, RowRecord, TransactionRecord, Tx};
    use std::collections::HashMap;

    pub mod transactions {
        use super::funds;
        use super::{funds::ClientFunds, Client, HashMap, RowRecord, TransactionRecord, Tx};
        use crate::{accounts::TxType, funds::FundingStates};

        #[derive(Debug, PartialEq, Hash, Eq, Copy, Clone)]
        pub struct LogKey(pub Tx);

        pub type TransactionLog = HashMap<LogKey, TransactionRecord>;

        pub fn valid_resolve(current: &TransactionRecord, client_funds: &ClientFunds) -> bool {
            if let Some(n) = client_funds.get(&current.client) {
                match n {
                    FundingStates::Disputed(_) => true,
                    _ => false,
                }
            } else {
                false
            }
        }

        pub fn valid_transaction(
            current: &TransactionRecord,
            logs: &TransactionLog,
            client_funds: &ClientFunds,
        ) -> bool {
            match current.r#type {
                TxType::Dispute => logs.contains_key(&LogKey(current.tx)),
                TxType::Resolve | TxType::Chargeback => valid_resolve(current, client_funds),
                _ => true,
            }
        }

        fn should_replace(current: &TransactionRecord) -> bool {
            match current.r#type {
                TxType::Dispute | TxType::Resolve | TxType::Chargeback => false,
                _ => true,
            }
        }

        pub fn insert(map: &mut TransactionLog, record: RowRecord, client_funds: &mut ClientFunds) {
            let key = LogKey(record.tx);
            let current = TransactionRecord::from(record);
            if valid_transaction(&current, map, client_funds) {
                funds::insert(client_funds, &current);
                if should_replace(&current) {
                    map.insert(key, current);
                }
            }
        }
    }

    pub mod funds {
        use super::{Client, HashMap, TransactionRecord};
        use crate::funds::FundingStates;
        pub type ClientFunds = HashMap<Client, FundingStates>;

        pub fn insert(map: &mut ClientFunds, record: &TransactionRecord) {
            let client = record.client;
            let funds;
            if map.contains_key(&client) {
                funds = map.get(&client).unwrap().transact(record);
            } else {
                funds = FundingStates::new(record.amount, record.tx);
            }
            map.insert(client, funds);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use records::transactions;

    use super::records::funds;
    use super::records::funds::ClientFunds;
    use super::records::transactions::{LogKey, TransactionLog};
    use crate::{
        amount::{Amount, RecordFloatAmount},
        funds::FundingStates,
        funds::Funds,
    };

    use super::{records, Client, RowRecord, TransactionRecord, Tx, TxType};

    #[test]
    fn insert_record_to_map() {
        let record = RowRecord {
            client: Client(1234),
            tx: Tx(556),
            amount: RecordFloatAmount(100.0),
            r#type: TxType::Deposit,
        };
        let mut map = TransactionLog::new();
        assert!(map.is_empty());
        let mut client_funds = ClientFunds::new();
        records::transactions::insert(&mut map, record, &mut client_funds);
        assert!(!map.is_empty());
        assert!(map.contains_key(&LogKey(Tx(556))));

        assert_eq!(
            map.get(&LogKey(Tx(556))).unwrap(),
            &TransactionRecord {
                tx: Tx(556),
                amount: Amount::new(1000000),
                r#type: TxType::Deposit,
                client: Client(1234)
            }
        )
    }
    #[test]
    fn insert_new_funds() {
        let mut map = ClientFunds::new();
        let record = TransactionRecord {
            tx: Tx(556),
            amount: Amount::new(1000),
            r#type: TxType::Deposit,
            client: Client(1234),
        };
        funds::insert(&mut map, &record);
        assert!(map.contains_key(&record.client));
        let fund = map.get(&record.client).unwrap();
        assert_eq!(
            fund,
            &FundingStates::Valid(Funds::new(Amount::new(1000), record.tx))
        );
    }

    #[test]
    fn three_deposits() {
        let mut map = ClientFunds::new();
        let records = vec![
            TransactionRecord {
                tx: Tx(1),
                amount: Amount::new(10000),
                r#type: TxType::Deposit,
                client: Client(1),
            },
            TransactionRecord {
                tx: Tx(2),
                amount: Amount::new(20000),
                r#type: TxType::Deposit,
                client: Client(2),
            },
            TransactionRecord {
                tx: Tx(3),
                amount: Amount::new(20000),
                r#type: TxType::Deposit,
                client: Client(1),
            },
        ];
        records
            .iter()
            .for_each(|record| funds::insert(&mut map, &record));
        assert_eq!(
            map.get(&Client(1)).unwrap(),
            &FundingStates::Valid(Funds::new(Amount::new(30000), Tx(3)))
        );
        assert_eq!(
            map.get(&Client(2)).unwrap(),
            &FundingStates::Valid(Funds::new(Amount::new(20000), Tx(2)))
        );
    }

    #[test]
    fn not_existing() {
        let record = RowRecord {
            tx: Tx(1),
            amount: RecordFloatAmount(100.0),
            r#type: TxType::Deposit,
            client: Client(1),
        };
        let bad_dispute = RowRecord {
            tx: Tx(2),
            amount: RecordFloatAmount(100.0),
            r#type: TxType::Dispute,
            client: Client(1),
        };
        let mut transaction_log = TransactionLog::new();
        let mut client_funds = ClientFunds::new();
        records::transactions::insert(&mut transaction_log, record, &mut client_funds);
        assert!(!transaction_log.is_empty());
        assert!(!client_funds.is_empty());
        records::transactions::insert(&mut transaction_log, bad_dispute, &mut client_funds);
        let xx = client_funds.get(&Client(1)).unwrap();
        assert_eq!(
            xx,
            &FundingStates::Valid(Funds::new(Amount::new(1000000), Tx(1)))
        )
    }

    #[test]
    fn dispute_transaction() {
        let records = vec![
            RowRecord {
                tx: Tx(1),
                amount: RecordFloatAmount(1.0),
                r#type: TxType::Deposit,
                client: Client(1),
            },
            RowRecord {
                tx: Tx(2),
                amount: RecordFloatAmount(2.0),
                r#type: TxType::Deposit,
                client: Client(2),
            },
            RowRecord {
                tx: Tx(1),
                amount: RecordFloatAmount(0.0),
                r#type: TxType::Dispute,
                client: Client(1),
            },
        ];
        let mut transaction_log = TransactionLog::new();
        let mut client_funds = ClientFunds::new();
        records
            .iter()
            .for_each(|record| records::transactions::insert(&mut transaction_log, *record, &mut client_funds));
        
        // let xx = .unwrap();
        if let Some(fund) = client_funds.get(&Client(1)) {
            match fund {
                FundingStates::Disputed(n) => assert_eq!(n.held, Amount::new(10000)),
                _ => assert_eq!(true, false)
            }
        } else {
           assert_eq!(true, false)
        }
    }
}
