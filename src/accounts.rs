use std::ops;


type PaymentResult<T> = Result<T, &'static str>;

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd)]
struct Amount(f64);

impl Amount {
    fn from_f64(n: f64) -> PaymentResult<Amount> {
        if n >= 0.0001 {
            Ok(Amount(n))
        } else {
            Err("An Amount cannot be smaller than 0.0001")
        }
    }
}

impl ops::Add<Amount> for Amount {
    type Output = Self;

    fn add(self, _rhs: Amount) -> Amount {
        Amount(self.0 + _rhs.0)
    }
}

impl ops::Sub<Amount> for Amount {
    type Output = Self;

    fn sub(self, _rhs: Amount) -> Amount {
        if self >= _rhs {
            return Amount(self.0 - _rhs.0)
        }
        self
    }
}


#[derive(Debug)]
struct Client(u16);

#[derive(Debug)]
struct Tx(u32);

#[derive(Debug)]
struct Output {
    client: Client,
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

#[derive(Debug)]
enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Debug)]
struct Transaction<'a>{
    action:&'a TxType,
    client:&'a Client,
    tx: &'a Tx,
    amount:Amount
}


#[derive(Debug)]
struct Account<'a>{
    value: Amount,
    client:&'a Client
}



impl <'a> Account<'a> {
    fn transact(&self,  transaction:&Transaction){
        // match transaction.action {
        //     TxType::Deposit => 
        // }

    }
    fn deposit(&mut self, transaction:&Transaction){
        self.value = self.value + transaction.amount
    }

    fn withdraw(&mut self, transaction:&Transaction){
        self.value = self.value - transaction.amount
    }

}



#[cfg(test)]
mod tests {
    use super::{Amount, Transaction, TxType, Tx, Client, Account};

    #[test]
    fn four_precision() {
        let floor = 0.0001;
        assert_eq!(Amount::from_f64(floor).unwrap().0, floor);

        let under_floor = floor - 0.000000000001;
        assert_eq!(
            Amount::from_f64(under_floor).unwrap_err(),
            "An Amount cannot be smaller than 0.0001"
        );
    }

    #[test]
    fn addition_for_amount() {
        let a = Amount(1.0);
        let b = Amount(1.5);
        assert_eq!(Amount(2.5), a + b);
        assert_eq!(Amount(2.5), b + a );
        assert_eq!(Amount(10.0), Amount(10.0) + Amount(0.0));
    }

    #[test]
    fn sub_for_amount() {
        assert_eq!(Amount(2.5), Amount(5.0) - Amount(2.5));
        assert_eq!(Amount(2.5), Amount(2.5) - Amount(3.0));
        assert_eq!(Amount(0.0), Amount(10.0) - Amount(10.000));
    }

    #[test]
    fn a_deposit_increases_accounts_value() {
        let client = Client(1);
        let mut account = Account{client:&client, value:Amount(0.5)};
        let transaction = Transaction{action:&TxType::Deposit, client:&client, amount:Amount(1.0), tx:&Tx(1)};
        account.deposit(&transaction);
        assert_eq!(account.value, Amount(1.5));
    }
}