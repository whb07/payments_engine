use crate::amount::{Amount};


trait Funding {
    fn new(deposit:Amount) -> Self;
    fn total(&self) -> Amount;
}

struct Funds {
    held:Amount,
    available:Amount
}


impl Funding for Funds {
    fn new(deposit: Amount) -> Funds{
        Funds { available: deposit, held: Amount::new(0)}
    }
    fn total(&self) -> Amount {
        self.available + self.held
    }
}


#[cfg(test)]
mod tests {
    use super::{Funds, Amount, Funding};
    #[test]
    fn new_fund() {
        let amount = Amount::new(100);
        let funds = Funds::new(amount);
        assert_eq!(amount, funds.available);
        assert_eq!(Amount::new(0), funds.held);
        assert_eq!(amount, funds.total());
    }

    #[test]
    fn total() {
        let amount = Amount::new(100);
        let funds = Funds{available:amount, held:Amount::new(23)};
        assert_eq!(Amount::new(123), funds.total());
    }
}
