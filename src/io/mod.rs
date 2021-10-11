use futures::future::join_all;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader};
use std::sync::Arc;

use dashmap::DashMap;
use tokio::task::JoinHandle;

use crate::processor::{self, Client};
use crate::transactions::{Transaction, TransactionWithStatus};

pub async fn read_csv(filename: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let client_db = Arc::new(DashMap::<u16, Client>::new());
    let transactions_db = Arc::new(DashMap::<u32, TransactionWithStatus>::new());

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(reader);

    let mut transactions: Vec<JoinHandle<()>> = vec![];

    for result in reader.deserialize() {
        let tx: Transaction = result?;
        let client_db = client_db.clone();
        let transactions_db = transactions_db.clone();

        transactions.push(tokio::task::spawn(async move {
            processor::handle_transaction(tx, &client_db, &transactions_db).await;
        }));
    }

    join_all(transactions).await;

    write_csv(&client_db);
    Ok(())
}

pub fn write_csv(clients_db: &Arc<DashMap<u16, Client>>) {
    let mut writer = csv::Writer::from_writer(io::stdout());
    clients_db.iter().for_each(|client| {
        writer.serialize(*client);
    });
    writer.flush();
}
