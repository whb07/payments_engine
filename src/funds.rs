use crate::amount::Amount;
use crate::transactions::Client;

#[derive(Debug, PartialEq, Eq)]
pub enum FundingStates {
    Valid,
    Disputed,
    Frozen,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Funds {
    pub held: Amount,
    pub available: Amount,
    pub client: Client,
    pub state: FundingStates,
}

impl Funds {
    pub fn total(&self) -> Amount {
        self.available + self.held
    }
    pub fn deposit(&mut self, amount: Amount) {
        if not_frozen(&self) {
            self.available = self.available + amount
        }
    }

    pub fn withdraw(&mut self, amount: Amount) {
        if not_frozen(&self) {
            self.available = self.available - amount
        }
    }

    pub fn dispute(&mut self, amount: Amount) {
        if not_frozen(&self) {
            self.held = self.held + amount;
            self.available = self.available - amount;
            self.update_dispute();
        }
    }

    pub fn resolve(&mut self, amount: Amount) {
        if not_frozen(&self) && self.state == FundingStates::Disputed {
            self.held = self.held - amount;
            self.available = self.available + amount;
            self.update_dispute();
        }
    }

    fn update_dispute(&mut self) -> bool {
        if self.held.0 > 0 {
            self.state = FundingStates::Disputed;
            true
        } else {
            self.state = FundingStates::Valid;
            false
        }
    }

    pub fn chargeback(&mut self, amount: Amount) {
        if not_frozen(&self) && self.state == FundingStates::Disputed {
            self.held = self.held - amount;
            self.state = FundingStates::Frozen;
        }
    }
}

pub fn not_frozen(fund: &Funds) -> bool {
    fund.state != FundingStates::Frozen
}

#[cfg(test)]
mod tests {
    use super::{Amount, Client, FundingStates, Funds};
    #[test]
    fn test_fund_total() {
        let mut fund = Funds {
            state: FundingStates::Disputed,
            available: Amount::new(1000),
            held: Amount::new(1000),
            client: Client(1),
        };
        assert_eq!(fund.total(), Amount::new(2000));
        fund.available = Amount::new(555);
        fund.held = Amount::new(0);
        assert_eq!(fund.total(), Amount::new(555));
    }
    #[test]
    fn test_deposit() {
        let mut fund = Funds {
            state: FundingStates::Disputed,
            available: Amount::new(0),
            held: Amount::new(0),
            client: Client(1),
        };
        assert_eq!(fund.total(), Amount::new(0));
        fund.deposit(Amount::new(100));
        assert_eq!(fund.total(), Amount::new(100));
    }
    #[test]
    fn test_withdrawal() {
        let mut fund = Funds {
            state: FundingStates::Disputed,
            available: Amount::new(0),
            held: Amount::new(0),
            client: Client(1),
        };
        assert_eq!(fund.total(), Amount::new(0));
        // withdraw some money thats beyond our 0 balance
        fund.withdraw(Amount::new(100));
        assert_eq!(fund.total(), Amount::new(0));

        fund.deposit(Amount::new(250));
        fund.withdraw(Amount::new(25));
        assert_eq!(fund.total(), Amount::new(225));
        assert_eq!(fund.available, Amount::new(225));
        assert_eq!(fund.held, Amount::new(0));
        // withdraw again beyond our limit
        fund.withdraw(Amount::new(300));
        assert_eq!(fund.total(), Amount::new(225));
    }
    #[test]
    fn test_dispute() {
        let mut fund = Funds {
            state: FundingStates::Disputed,
            available: Amount::new(0),
            held: Amount::new(0),
            client: Client(1),
        };
        assert_eq!(fund.total(), Amount::new(0));
        // withdraw some money thats beyond our 0 balance
        fund.withdraw(Amount::new(100));
        assert_eq!(fund.total(), Amount::new(0));

        fund.deposit(Amount::new(250));
        fund.withdraw(Amount::new(25));
        assert_eq!(fund.total(), Amount::new(225));
        assert_eq!(fund.available, Amount::new(225));
        assert_eq!(fund.held, Amount::new(0));
        // withdraw again beyond our limit
        fund.withdraw(Amount::new(300));
        assert_eq!(fund.total(), Amount::new(225));
    }
    #[test]
    fn test_resolve() {
        let mut fund = Funds {
            state: FundingStates::Disputed,
            available: Amount::new(100),
            held: Amount::new(20),
            client: Client(1),
        };
        fund.resolve(Amount::new(19));
        assert_eq!(fund.available, Amount::new(119));
        assert_eq!(fund.total(), Amount::new(120));
        assert_eq!(fund.state, FundingStates::Disputed);
        fund.resolve(Amount::new(1));
        assert_eq!(fund.available, Amount::new(120));
        assert_eq!(fund.state, FundingStates::Valid);
    }

    #[test]
    fn test_resolve_on_valid() {
        let mut fund = Funds {
            state: FundingStates::Valid,
            available: Amount::new(100),
            held: Amount::new(0),
            client: Client(1),
        };
        fund.resolve(Amount::new(20));
        assert_eq!(fund.available, Amount::new(100));
        assert_eq!(fund.held, Amount::new(0));
        assert_eq!(fund.state, FundingStates::Valid);
        fund.state = FundingStates::Disputed;
        fund.resolve(Amount::new(20));
        assert_eq!(fund.held, Amount::new(0));
        assert_eq!(fund.state, FundingStates::Valid);
    }

    #[test]
    fn test_chargeback() {
        let mut fund = Funds {
            state: FundingStates::Disputed,
            available: Amount::new(100),
            held: Amount::new(20),
            client: Client(1),
        };
        fund.chargeback(Amount::new(5));
        assert_eq!(fund.state, FundingStates::Frozen);
        assert_eq!(fund.held, Amount::new(15));
        assert_eq!(fund.total(), Amount::new(115));
        // run it again
        fund.chargeback(Amount::new(1));
        assert_eq!(fund.total(), Amount::new(115));
        assert_eq!(fund.held, Amount::new(15));
    }
}
