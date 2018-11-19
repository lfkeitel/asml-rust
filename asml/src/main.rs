extern crate srecord;

use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: asml FILE");
        std::process::exit(1);
    }

    let srec_path = Path::new(&args[1]);

    println!("Reading srecord file {}", srec_path.display());

    let records = srecord::parse_file(srec_path).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    for r in &records {
        println!("{}", r);
    }
}
