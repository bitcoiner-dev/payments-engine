use std::cmp::Eq;

use serde::Deserialize;

#[derive(Copy, Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    #[serde(rename = "amount")]
    pub amount: Option<f64>,
}

impl Transaction {
    #[cfg(test)]
    pub fn new_deposit(client_id: u16, tx_id: u32, amount: f64) -> Self {
        Self {
            tx_type: TransactionType::Deposit,
            client_id,
            tx_id,
            amount: Some(amount),
        }
    }

    #[cfg(test)]
    pub fn new_withdrawal(client_id: u16, tx_id: u32, amount: f64) -> Self {
        Self {
            tx_type: TransactionType::Withdrawal,
            client_id,
            tx_id,
            amount: Some(amount),
        }
    }

    #[cfg(test)]
    pub fn new_dispute(client_id: u16, tx_id: u32) -> Self {
        Self {
            tx_type: TransactionType::Dispute,
            client_id,
            tx_id,
            amount: None,
        }
    }

    #[cfg(test)]
    pub fn new_resolve(client_id: u16, tx_id: u32) -> Self {
        Self {
            tx_type: TransactionType::Resolve,
            client_id,
            tx_id,
            amount: None,
        }
    }

    #[cfg(test)]
    pub fn new_chargeback(client_id: u16, tx_id: u32) -> Self {
        Self {
            tx_type: TransactionType::Chargeback,
            client_id,
            tx_id,
            amount: None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TransactionStatus {
    Good,
    Disputed,
    Chargeback,
}

#[derive(Copy, Clone, Debug)]
pub struct TransactionWithStatus {
    pub tx: Transaction,
    pub status: TransactionStatus,
}
