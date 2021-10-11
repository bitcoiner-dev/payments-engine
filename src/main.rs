use payments_engine::io;
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: ");
        println!("\t{} transactions.csv", args[0]);
        process::exit(1);
    }

    // In a "real" setting, we will be fed this data through a socket.
    // Therefore, use async task here to handle that within an async task
    // A new task will be spawned when new transactions are posted.
    io::read_csv(&args[1])
        .await
        .expect("Error reading CSV file");
}
