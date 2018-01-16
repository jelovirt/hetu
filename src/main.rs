extern crate hetu;

use hetu::Ssn;
use hetu::SsnPattern;
use hetu::ParseError;
use hetu::ErrorIndexRange;
use std::env;
use std::process;

pub fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() > 1 && &args[0] == "-p" {
        match SsnPattern::parse(&args[1]) {
            Err(ref err) => {
                eprintln!("Error: {}\n\n  {}\n  {}", err, &args[1], index_arrows(err));
                process::exit(1)
            }
            Ok(pattern) => println!("{}", Ssn::generate_by_pattern(&pattern))
        }
    } else if args.len() == 0 {
        let pattern = SsnPattern::new();
        println!("{}", Ssn::generate_by_pattern(&pattern));
    } else {
        let ssn: &String = &args[0];
        match Ssn::parse(&ssn) {
            Ok(_) => (),
            Err(ref err) => {
                eprintln!("Error: {}\n\n  {}\n  {}", err, &ssn, index_arrows(err));
                process::exit(1)
            }
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
