extern crate hetu;
extern crate ansi_term;

use hetu::ErrorIndexRange;
use hetu::ParseError;
use hetu::Ssn;
use hetu::SsnPattern;
use std::env;
use std::io::{self, BufRead};
use std::process;
use ansi_term::Colour::Red;

pub fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && (&args[0] == "-h" || &args[0] == "--help") {
        help();
        return;
    }

    if args.len() > 1 && &args[0] == "-p" {
        match SsnPattern::parse(&args[1]) {
            Err(ref err) => {
                eprintln!("Error: {}\n\n  {}\n  {}", err, &args[1], Red.paint(index_arrows(err)));
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
    } else if args.len() == 1 && &args[0] == "-" {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            parse(&(line.unwrap()));
        }
    } else if args.is_empty() {
        let pattern = SsnPattern::new();
        match Ssn::generate_by_pattern(&pattern) {
            Ok(ref ssn) => println!("{}", ssn),
            Err(ref err) => {
                eprintln!("Error: {}", err);
                process::exit(1)
            }
        }
    } else {
        parse(&args[0]);
    }
}

fn parse(ssn: &str) -> () {
    match Ssn::parse(&ssn) {
        Ok(_) => (),
        Err(ref err) => {
            eprintln!("Error: {}\n\n  {}\n  {}", err, &ssn, Red.paint(index_arrows(err)));
            process::exit(1)
        }
    }
}

fn help() {
    println!(
        "Validator and generator for Finnish SSN

Usage:
    hetu <ssn>
    hetu -
    hetu -p <pattern>
    hetu [options]

Options:
    -h, --help          Display this message
"
    );
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
