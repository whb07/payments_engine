type PaymentResult<T> = Result<T, &'static str>;

#[derive(Debug)]
struct Amount(f64);

impl Amount {
    fn new(n: f64) -> PaymentResult<Amount> {
        if n >= 0.0001 {
            Ok(Amount(n))
        } else {
            Err("An Amount cannot be smaller than 0.0001")
        }
    }
}

// enum Transactions {
//     Deposit,
//     Withdrawal,
//     Dispute,
//     Resolve,
//     Chargeback,
// }

#[derive(Debug)]
struct Client(u16);

#[derive(Debug)]
struct TX(u32);

#[derive(Debug)]
struct Output {
    client: Client,
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

#[cfg(test)]
mod tests {
    use super::Amount;

    #[test]
    fn four_precision() {
        let floor = 0.0001;
        assert_eq!(Amount::new(floor).unwrap().0, floor);

        let under_floor = floor - 0.000000000001;
        assert_eq!(
            Amount::new(under_floor).unwrap_err(),
            "An Amount cannot be smaller than 0.0001"
        );
    }
}
