use serde::{Deserialize, Serialize};
use std::ops;
use std::str::FromStr;

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Serialize, Deserialize, Eq)]
pub struct Amount(pub u64);

#[derive(Debug, PartialEq, Copy, Clone, PartialOrd, Serialize, Deserialize, Eq)]
pub struct ProcessedAmount(pub u64);

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

impl Amount {
    pub fn new(n: u64) -> Amount {
        Amount(n)
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
