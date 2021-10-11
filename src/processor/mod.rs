use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dashmap::DashMap;
use serde::{Serialize, Serializer};

use crate::transactions::{Transaction, TransactionStatus, TransactionType, TransactionWithStatus};

pub type TransactionsDb = Arc<DashMap<u32, TransactionWithStatus>>;
pub type ClientDb = Arc<DashMap<u16, Client>>;

#[derive(Copy, Clone, Serialize)]
pub struct Client {
    #[serde(rename = "client")]
    id: u16,
    #[serde(serialize_with = "change_precision")]
    available: f64,
    #[serde(serialize_with = "change_precision")]
    held: f64,
    #[serde(serialize_with = "change_precision")]
    total: f64,
    locked: bool,
}

fn change_precision<S>(amount: &f64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&format!("{:.4}", amount))
}

impl Client {
    fn new(id: u16) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            id: 0,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
}

impl Hash for Client {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

fn insert_new_transaction(tx: Transaction, tx_db: &TransactionsDb) {
    if !tx_db.contains_key(&tx.tx_id) {
        tx_db.insert(
            tx.tx_id,
            TransactionWithStatus {
                tx,
                status: TransactionStatus::Good,
            },
        );
    }
}

pub async fn handle_transaction(tx: Transaction, client_db: &ClientDb, tx_db: &TransactionsDb) {
    if tx_db.contains_key(&tx.tx_id) {
        // Transaction IDs are globally unique, ignore an incoming
        // transaction that has the same transaction type and ID as
        // an existing transaction
        let existing_tx = *(tx_db.get(&tx.tx_id).unwrap());
        if existing_tx.tx.tx_type == tx.tx_type && existing_tx.tx.tx_id == tx.tx_id {
            return;
        }
    }

    match tx.tx_type {
        TransactionType::Deposit => {
            if let Some(amount) = tx.amount {
                insert_new_transaction(tx, tx_db);
                let mut client = client_db
                    .entry(tx.client_id)
                    .or_insert(Client::new(tx.client_id));
                client.available += amount;
                client.total += amount;
            }
        }
        TransactionType::Withdrawal => {
            if let Some(amount) = tx.amount {
                let mut client = client_db
                    .entry(tx.client_id)
                    .or_insert(Client::new(tx.client_id));
                if client.available >= amount {
                    insert_new_transaction(tx, tx_db);
                    client.available -= amount;
                    client.total -= amount;
                }
            }
        }
        TransactionType::Dispute => {
            if !client_db.contains_key(&tx.client_id) {
                return;
            }

            if let Some(mut disputed_tx) = tx_db.get_mut(&tx.tx_id) {
                if let TransactionStatus::Good = disputed_tx.status {
                    let id = tx.client_id;
                    let mut client = client_db.get_mut(&id).unwrap();
                    client.available -= disputed_tx.tx.amount.unwrap();
                    client.held += disputed_tx.tx.amount.unwrap();
                    disputed_tx.status = TransactionStatus::Disputed;
                }
            }
        }
        TransactionType::Resolve => {
            if !client_db.contains_key(&tx.client_id) {
                return;
            }

            if let Some(mut resolved_tx) = tx_db.get_mut(&tx.tx_id) {
                if let TransactionStatus::Disputed = resolved_tx.status {
                    if let Some(resolved_amount) = resolved_tx.tx.amount {
                        let id = tx.client_id;
                        let mut client = client_db.get_mut(&id).unwrap();

                        if client.held >= resolved_amount {
                            client.available += resolved_amount;
                            client.held -= resolved_amount;
                            resolved_tx.status = TransactionStatus::Good;
                        }
                    }
                }
            }
        }
        TransactionType::Chargeback => {
            if !client_db.contains_key(&tx.client_id) {
                return;
            }

            if let Some(mut chargeback_tx) = tx_db.get_mut(&tx.tx_id) {
                if let TransactionStatus::Disputed = chargeback_tx.status {
                    if let Some(chargeback_amount) = chargeback_tx.tx.amount {
                        let id = tx.client_id;
                        let mut client = client_db.get_mut(&id).unwrap();

                        if client.held >= chargeback_amount {
                            client.held -= chargeback_amount;
                            client.total -= chargeback_amount;
                            client.locked = true;
                            chargeback_tx.status = TransactionStatus::Chargeback;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (ClientDb, TransactionsDb) {
        (
            Arc::new(DashMap::<u16, Client>::new()),
            Arc::new(DashMap::<u32, TransactionWithStatus>::new()),
        )
    }

    #[tokio::test]
    async fn test_deposit() {
        let (client_db, transactions_db) = setup();

        let tx = Transaction::new_deposit(1, 1, 3.0);

        handle_transaction(tx, &client_db, &transactions_db).await;

        let client = client_db.get(&1).unwrap();

        assert_eq!(client.available, 3.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 3.0);
        assert!(!client.locked);
    }

    #[tokio::test]
    async fn test_multiple_deposits_with_different_tx_ids_succeed() {
        let (client_db, transactions_db) = setup();

        let deposit1 = Transaction::new_deposit(1, 1, 3.0);
        let deposit2 = Transaction::new_deposit(1, 2, 2.0);

        handle_transaction(deposit1, &client_db, &transactions_db).await;
        handle_transaction(deposit2, &client_db, &transactions_db).await;

        let client = client_db.get(&1).unwrap();

        assert_eq!(client.available, 5.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 5.0);
        assert!(!client.locked);
    }

    #[tokio::test]
    async fn test_multiple_deposits_with_different_client_ids_succeed() {
        let (client_db, transactions_db) = setup();

        let deposit1 = Transaction::new_deposit(1, 3, 3.0);
        let deposit2 = Transaction::new_deposit(2, 4, 2.0);

        handle_transaction(deposit1, &client_db, &transactions_db).await;
        handle_transaction(deposit2, &client_db, &transactions_db).await;

        let client = client_db.get(&1).unwrap();

        assert_eq!(client.available, 3.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 3.0);
        assert!(!client.locked);

        let client = client_db.get(&2).unwrap();

        assert_eq!(client.available, 2.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 2.0);
        assert!(!client.locked);
    }

    #[tokio::test]
    async fn test_multiple_deposits_with_same_tx_ids_allows_only_first() {
        let (client_db, transactions_db) = setup();

        let deposit1 = Transaction::new_deposit(1, 1, 3.0);
        let deposit2 = Transaction::new_deposit(1, 1, 2.0);

        handle_transaction(deposit1, &client_db, &transactions_db).await;
        handle_transaction(deposit2, &client_db, &transactions_db).await;

        let client = client_db.get(&1).unwrap();

        assert_eq!(client.available, 3.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 3.0);
        assert!(!client.locked);
    }

    #[tokio::test]
    async fn test_deposit_and_withdrawal() {
        let (client_db, transactions_db) = setup();

        let deposit = Transaction::new_deposit(1, 1, 3.0);
        let withdrawal = Transaction::new_withdrawal(1, 2, 1.5);

        handle_transaction(deposit, &client_db, &transactions_db).await;
        handle_transaction(withdrawal, &client_db, &transactions_db).await;

        let client = client_db.get(&1).unwrap();

        assert_eq!(client.available, 1.5);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 1.5);
        assert!(!client.locked);
    }

    #[tokio::test]
    async fn test_withdrawing_more_than_available_fails() {
        let (client_db, transactions_db) = setup();

        let deposit = Transaction::new_deposit(1, 1, 3.0);
        let withdrawal = Transaction::new_withdrawal(1, 2, 4.0);

        handle_transaction(deposit, &client_db, &transactions_db).await;
        handle_transaction(withdrawal, &client_db, &transactions_db).await;

        let client = client_db.get(&1).unwrap();

        assert_eq!(client.available, 3.0);
        assert_eq!(client.held, 0.0);
        assert_eq!(client.total, 3.0);
        assert!(!client.locked);
    }

    #[tokio::test]
    async fn test_disputing_an_existing_transaction_succeeds() {
        let (client_db, transactions_db) = setup();

        let deposit = Transaction::new_deposit(1, 1, 3.0);
        let dispute = Transaction::new_dispute(1, 1);

        handle_transaction(deposit, &client_db, &transactions_db).await;
        handle_transaction(dispute, &client_db, &transactions_db).await;

        let client = client_db.get(&1).unwrap();

        assert_eq!(client.available, 0.0);
        assert_eq!(client.held, 3.0);
        assert_eq!(client.total, 3.0);
        assert!(!client.locked);
        assert_eq!(
            transactions_db.get(&1).unwrap().status,
            TransactionStatus::Disputed
        );
    }

    #[tokio::test]
    async fn test_dangling_dispute_is_ignored() {
        let (client_db, transactions_db) = setup();

        let dispute = Transaction::new_dispute(1, 1);

        handle_transaction(dispute, &client_db, &transactions_db).await;

        assert!(client_db.get(&1).is_none());
        assert!(transactions_db.get(&1).is_none());
    }

    #[tokio::test]
    async fn test_resolving_a_disputed_transaction_succeeds() {
        let (client_db, transactions_db) = setup();

        let deposit1 = Transaction::new_deposit(1, 1, 3.0);
        let deposit2 = Transaction::new_deposit(1, 2, 1.0);
        let dispute = Transaction::new_dispute(1, 1);
        let resolve = Transaction::new_resolve(1, 1);

        handle_transaction(deposit1, &client_db, &transactions_db).await;
        handle_transaction(deposit2, &client_db, &transactions_db).await;

        assert_eq!(client_db.get(&1).unwrap().available, 4.0);

        handle_transaction(dispute, &client_db, &transactions_db).await;
        assert_eq!(client_db.get(&1).unwrap().available, 1.0);
        assert_eq!(client_db.get(&1).unwrap().held, 3.0);
        assert_eq!(
            transactions_db.get(&1).unwrap().status,
            TransactionStatus::Disputed
        );

        handle_transaction(resolve, &client_db, &transactions_db).await;
        assert_eq!(client_db.get(&1).unwrap().available, 4.0);
        assert_eq!(client_db.get(&1).unwrap().held, 0.0);
        assert_eq!(
            transactions_db.get(&1).unwrap().status,
            TransactionStatus::Good
        );
    }

    #[tokio::test]
    async fn test_dangling_resolve_is_ignored() {
        let (client_db, transactions_db) = setup();

        let dispute = Transaction::new_resolve(1, 1);

        handle_transaction(dispute, &client_db, &transactions_db).await;

        assert!(client_db.get(&1).is_none());
        assert!(transactions_db.get(&1).is_none());
    }

    #[tokio::test]
    async fn test_chargeback_succeeds() {
        let (client_db, transactions_db) = setup();

        let deposit1 = Transaction::new_deposit(1, 1, 3.0);
        let deposit2 = Transaction::new_deposit(1, 2, 1.0);
        let dispute = Transaction::new_dispute(1, 1);
        let chargeback = Transaction::new_chargeback(1, 1);

        handle_transaction(deposit1, &client_db, &transactions_db).await;
        handle_transaction(deposit2, &client_db, &transactions_db).await;
        assert_eq!(client_db.get(&1).unwrap().available, 4.0);

        handle_transaction(dispute, &client_db, &transactions_db).await;
        assert_eq!(client_db.get(&1).unwrap().available, 1.0);
        assert_eq!(client_db.get(&1).unwrap().held, 3.0);
        assert_eq!(
            transactions_db.get(&1).unwrap().status,
            TransactionStatus::Disputed
        );

        handle_transaction(chargeback, &client_db, &transactions_db).await;
        assert_eq!(client_db.get(&1).unwrap().available, 1.0);
        assert_eq!(client_db.get(&1).unwrap().held, 0.0);
        assert!(client_db.get(&1).unwrap().locked);
        assert_eq!(
            transactions_db.get(&1).unwrap().status,
            TransactionStatus::Chargeback
        );
    }

    #[tokio::test]
    async fn test_chargeback_for_a_non_disputed_transaction_is_ignored() {
        let (client_db, transactions_db) = setup();

        let deposit = Transaction::new_deposit(1, 1, 3.0);
        let chargeback = Transaction::new_chargeback(1, 1);

        handle_transaction(deposit, &client_db, &transactions_db).await;
        assert_eq!(client_db.get(&1).unwrap().available, 3.0);
        assert_eq!(client_db.get(&1).unwrap().held, 0.0);

        handle_transaction(chargeback, &client_db, &transactions_db).await;
        assert_eq!(client_db.get(&1).unwrap().available, 3.0);
        assert_eq!(client_db.get(&1).unwrap().held, 0.0);
        assert!(!client_db.get(&1).unwrap().locked);
        assert_eq!(
            transactions_db.get(&1).unwrap().status,
            TransactionStatus::Good
        );
    }

    #[tokio::test]
    async fn test_dangling_chargeback_is_ignored() {
        let (client_db, transactions_db) = setup();

        let dispute = Transaction::new_chargeback(1, 1);

        handle_transaction(dispute, &client_db, &transactions_db).await;

        assert!(client_db.get(&1).is_none());
        assert!(transactions_db.get(&1).is_none());
    }
}
