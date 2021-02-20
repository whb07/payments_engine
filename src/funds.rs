use crate::accounts::{TransactionRecord, Tx, TxType};
use crate::amount::Amount;

pub trait Funding {
    fn total(&self) -> Amount;
    fn deposit(&self, amount: Amount, tx: Tx) -> Self;
    fn withdraw(&self, amount: Amount, tx: Tx) -> Self;
    fn dispute(&self, disputed_record: &TransactionRecord, tx: Tx) -> Self;
    fn resolve(&self, disputed_record: &TransactionRecord, tx: Tx) -> Self;
    fn chargeback(&self, disputed_record: &TransactionRecord, tx: Tx) -> Self;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Funds {
    pub held: Amount,
    pub available: Amount,
    from: Tx,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FundingStates {
    Valid(Funds),
    Disputed(Funds),
    Frozen(Funds),
}

impl Funds {
    pub fn new(deposit: Amount, tx: Tx) -> Funds {
        Funds {
            available: deposit,
            held: Amount::new(0),
            from: tx,
        }
    }
    pub fn transact(&self, record: &TransactionRecord) -> Funds {
        match record.r#type {
            TxType::Deposit => self.deposit(record.amount, record.tx),
            TxType::Withdrawal => self.withdraw(record.amount, record.tx),
            TxType::Dispute => self.dispute(record, record.tx),
            TxType::Chargeback => self.chargeback(record, record.tx),
            TxType::Resolve => self.resolve(record, record.tx),
        }
    }
    fn do_nothing(&self) -> Funds {
        Funds {
            held: self.held,
            available: self.available,
            from: self.from,
        }
    }
}

impl FundingStates {
    pub fn new(deposit: Amount, tx: Tx) -> FundingStates {
        FundingStates::Valid(Funds::new(deposit, tx))
    }
    pub fn transact(&self, record: &TransactionRecord) -> FundingStates {
        match self {
            FundingStates::Frozen(fund) => FundingStates::Frozen(fund.do_nothing()),
            FundingStates::Disputed(fund) => {
                if record.r#type == TxType::Chargeback {
                    FundingStates::Frozen(fund.transact(record))
                } else {
                    FundingStates::Valid(fund.transact(record))
                }
            }
            FundingStates::Valid(fund) => {
                if record.r#type == TxType::Deposit || record.r#type == TxType::Withdrawal {
                    FundingStates::Valid(fund.transact(record))
                } else {
                    FundingStates::Disputed(fund.transact(record))
                }
            }
        }
    }
}

impl Funding for Funds {
    fn total(&self) -> Amount {
        self.available + self.held
    }

    fn deposit(&self, amount: Amount, tx: Tx) -> Funds {
        Funds {
            held: self.held,
            available: self.available + amount,
            from: tx,
        }
    }
    fn withdraw(&self, amount: Amount, tx: Tx) -> Funds {
        Funds {
            held: self.held,
            available: self.available - amount,
            from: tx,
        }
    }

    fn dispute(&self, disputed_record: &TransactionRecord, tx: Tx) -> Funds {
        Funds {
            held: self.held + disputed_record.amount,
            available: self.available - disputed_record.amount,
            from: tx,
        }
    }

    fn resolve(&self, disputed_record: &TransactionRecord, tx: Tx) -> Funds {
        Funds {
            held: Amount::new(0),
            available: self.total(),
            from: tx,
        }
    }

    fn chargeback(&self, disputed_record: &TransactionRecord, tx: Tx) -> Funds {
        Funds {
            held: Amount::new(0),
            available: self.total() - self.held,
            from: tx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Amount, Funding, Funds, Tx};
    #[test]
    fn new_fund() {
        let amount = Amount::new(100);
        let funds = Funds::new(amount, Tx(1));
        assert_eq!(amount, funds.available);
        assert_eq!(Amount::new(0), funds.held);
        assert_eq!(amount, funds.total());
    }

    #[test]
    fn total() {
        let amount = Amount::new(100);
        let funds = Funds {
            available: amount,
            held: Amount::new(23),
            from: Tx(123),
        };
        assert_eq!(Amount::new(123), funds.total());
    }
}
