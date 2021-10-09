use std::env;
use std::process;
use payments_engine::io;

fn main() {
  let args: Vec<String> = env::args().collect();
  
  if args.len() != 2 {
    println!("Usage: ");
    println!("\t{} transactions.csv", args[0]);
    process::exit(1);
  }

  io::read_csv(&args[1]).expect("Error reading CSV file");

}
