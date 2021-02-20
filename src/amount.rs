use serde::{Deserialize, Serialize};
use std::ops;
use std::str::FromStr;

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Serialize, Deserialize, Eq)]
pub struct Amount(u64);

impl Amount {
    pub fn new(n: u64) -> Amount {
        Amount(n)
    }
}

impl FromStr for Amount {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Amount, &'static str> {
        if let Some(index) = s.find('.') {
            let err: &'static str = "A valid amount is up to 4 digits precision";
            let decimal_length = (s.len() - 1) - index;
            if decimal_length > 4 {
                return Err(err);
            }
            let padding_amount = 4 - (decimal_length);
            let pad = "0".repeat(padding_amount);
            let mut nums: Vec<&str> = s.split('.').collect();
            let mut right_side = vec![nums.pop().unwrap()];
            right_side.push(&pad);
            let right = right_side.join("");
            match (nums[0].parse::<u64>(), right.parse::<u64>()) {
                (Ok(left), Ok(right)) => Ok(Amount((left * 10000) + right)),
                _ => Err(err),
            }
        } else {
            match s.parse::<u64>() {
                Ok(val) => Ok(Amount(val * 10000)),
                _ => Err("Bad input for amount"),
            }
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
            return Amount(self.0 - _rhs.0);
        }
        self
    }
}

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Serialize, Deserialize)]
pub struct RecordFloatAmount(pub f64);

impl From<RecordFloatAmount> for Amount {
    fn from(val: RecordFloatAmount) -> Amount {
        Amount::from_str(&val.0.to_string()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Amount;

    #[test]
    fn four_precision() {
        assert_eq!(Amount::from_str("100").unwrap().0, 1000000);
        assert_eq!(Amount::from_str("1.234").unwrap().0, 12340);
        assert_eq!(Amount::from_str("0.0001").unwrap().0, 1);
        assert_eq!(Amount::from_str("5.8").unwrap().0, 58000);
        assert_eq!(
            Amount::from_str("0.00001").unwrap_err(),
            "A valid amount is up to 4 digits precision"
        );
        assert_eq!(Amount::from_str("0.024").unwrap().0, 240);
    }

    #[test]
    fn addition_for_amount() {
        let a = Amount(1);
        let b = Amount(1);
        assert_eq!(Amount(2), a + b);
        assert_eq!(Amount(2), b + a);
        assert_eq!(Amount(10), Amount(10) + Amount(0));
        assert_eq!(Amount(0), Amount(0) + Amount(0));
    }

    #[test]
    fn sub_for_amount() {
        assert_eq!(Amount(25), Amount(50) - Amount(25));
        assert_eq!(Amount(25), Amount(25) - Amount(30));
        assert_eq!(Amount(0), Amount(10) - Amount(10));
    }
}
