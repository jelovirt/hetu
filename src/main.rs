extern crate hetu;

use hetu::ErrorIndexRange;
use hetu::ParseError;
use hetu::Ssn;
use hetu::SsnPattern;
use std::env;
use std::process;

pub fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && (&args[0] == "-h" || &args[0] == "--help") {
        println!(
            "Validator and generator for Finnish SSN

Usage:
    hetu <ssn>
    hetu -p <pattern>
    hetu [options]

Options:
    -h, --help          Display this message
"
        );
        return;
    }

    if args.len() > 1 && &args[0] == "-p" {
        match SsnPattern::parse(&args[1]) {
            Err(ref err) => {
                eprintln!("Error: {}\n\n  {}\n  {}", err, &args[1], index_arrows(err));
                process::exit(1)
            }
            Ok(pattern) => match Ssn::generate_by_pattern(&pattern) {
                Ok(ref ssn) => println!("{}", ssn),
                Err(ref err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1)
                }
            },
        }
    } else if args.len() == 0 {
        let pattern = SsnPattern::new();
        match Ssn::generate_by_pattern(&pattern) {
            Ok(ref ssn) => println!("{}", ssn),
            Err(ref err) => {
                eprintln!("Error: {}", err);
                process::exit(1)
            }
        }
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
