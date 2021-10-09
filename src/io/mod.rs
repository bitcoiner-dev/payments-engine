use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use crate::Transaction;

pub fn read_csv(filename: &str) -> Result<(), Box<dyn Error>> {
  let file = File::open(filename)?;
  let reader = BufReader::new(file);
  let mut reader = csv::ReaderBuilder::new()
      .trim(csv::Trim::All)
      .from_reader(reader);
  for result in reader.deserialize() {
    let tx: Transaction = result?;
    println!("{:?}", tx);
  }
  Ok(())
}
