extern crate hetu;

use hetu::Ssn;
use hetu::Gender;
use hetu::ParseError;
use hetu::ErrorIndexRange;
use std::env;

pub fn main() {
    if env::args().len() == 1 {
        println!("{}",
                 Ssn::generate_by_pattern(&Ssn {
                     year: 1976,
                     month: 10,
                     day: 25,
                     gender: Gender::Female,
                 }));
    } else {
        let ssn: String = env::args().skip(1).next().unwrap();
        match Ssn::parse(&ssn) {
            Ok(_) => (),
            Err(ref err) => println!("Error: {}\n\n  {}\n  {}", err, ssn, index_arrows(err)),
        }
    }
}

fn index_arrows(err: &ParseError) -> String {
    let mut res: String = String::new();
    for _ in 0..err.start() {
        res.push_str(" ");
    }
    for _ in err.start()..err.end() {
        res.push_str("^");
    }
    res
}
