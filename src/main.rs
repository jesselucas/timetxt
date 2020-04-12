extern crate timetxt;

use std::env;
use std::fs;
use timetxt::Time;

fn main() {
    let mut args = env::args();
    args.next(); // Skip program name arg

    let filename = match args.next() {
        Some(arg) => arg,
        None => panic!("Must provide filename"),
    };

    // Read file
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let t: Time = timetxt::parse_time(&contents).expect("Failed to parse");
    println!("{}", t);
}
