// use std::error::Error;
// use std::io;


// fn read() -> Result<(), Box<dyn Error>> {
//   let mut reader = csv::Reader::from_reader(io::stdin());
//   for rresult in reader.deserialize() {
//     let tx: Transaction = result?;
//     println!("{:?}", tx);
//   }
//   Ok(())
// }