use std::fmt;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
  Deposit,
  Withdrawal
}

// Amounts
#[derive(Debug, Deserialize)]
pub struct Amount(f64);

impl Amount {
  fn new(amount: f64) -> Amount {
    Self(amount)
  }
}

// Amounts have a precision of four decimal places
impl fmt::Display for Amount {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:.4}", self.0)
  }
}

// Transactions

#[derive(Debug, Deserialize)]
pub struct Transaction {
  #[serde(rename = "type")]
  tx_type: TransactionType,
  #[serde(rename = "client")]
  client_id: u16,
  #[serde(rename = "tx")]
  tx_id: u32,
  #[serde(rename = "amount")]
  amount: Amount
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn serialized_amount_has_four_decimal_places() {
    let num = 12.345678;
    let amount = Amount::new(num);
    let displayed_amount = format!("{}", amount);
    let expected_amount = format!("{:.4}", num);

    assert_eq!(displayed_amount, expected_amount, "Amount precision must be four decimal places");
  }

}