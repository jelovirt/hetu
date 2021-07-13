extern crate ansi_term;
extern crate hetu;

use ansi_term::Colour::Red;
use hetu::ErrorIndexRange;
use hetu::ParseError;
use hetu::Ssn;
use hetu::SsnPattern;
use std::env;
use std::io::{self, BufRead};
use std::process;

pub fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() == 1 && (&args[0] == "-h" || &args[0] == "--help") {
        help();
        return;
    }

    if args.len() == 2 && (&args[0] == "-p" || &args[0] == "--pattern") {
        generate(&args[1]);
    } else if args.len() == 1 && args[0].starts_with("--pattern=") {
        generate(&args[0][10..]);
    } else if args.len() == 1 && &args[0] == "-" {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            parse(&(line.unwrap()));
        }
    } else if args.is_empty() {
        generate_and_print(&SsnPattern::new());
    } else {
        parse(&args[0]);
    }
}

fn generate(pattern: &str) {
    match SsnPattern::parse(pattern) {
        Err(ref err) => {
            eprintln!(
                "Error: {}\n\n  {}\n  {}",
                err,
                pattern,
                Red.paint(index_arrows(err))
            );
            process::exit(1)
        }
        Ok(pattern) => generate_and_print(&pattern),
    }
}

fn generate_and_print(pattern: &SsnPattern) {
    match Ssn::generate_by_pattern(&pattern) {
        Ok(ref ssn) => println!("{}", ssn),
        Err(ref err) => {
            eprintln!("Error: {}", err);
            process::exit(1)
        }
    }
}

fn parse(ssn: &str) {
    match Ssn::parse(&ssn) {
        Ok(_) => (),
        Err(ref err) => {
            eprintln!(
                "Error: {}\n\n  {}\n  {}",
                err,
                &ssn,
                Red.paint(index_arrows(err))
            );
            process::exit(1)
        }
    }
}

fn help() {
    println!(
        "Validator and generator for Finnish Personal Identity Code (HETU).

Usage:
    hetu <HETU>         Validate argument HETU.
    hetu -              Read input from standard input and validate.
    hetu                Generate HETU.
    hetu -p <PATTERN>   Generate HETU using pattern.

Options:
    -h, --help
            Print this help message.
    -p, --pattern <pattern>
            Generate HETU by pattern. Patterns use a question mark ('?') for wildcard and wildcards
            can appear at any location in the pattern.

Arguments:
    <HETU>
            Validate argument HETU. Use a dash ('-') to read from standard input.

Examples:
    * Validate HETU:

        $ hetu 291269-2763

    * Generate HETU by pattern:

        $ hetu -p '291269-????'
        291269-7767
"
    );
}

fn index_arrows(err: &ParseError) -> String {
    let mut res: String = String::new();
    for _ in 0..err.start() {
        res.push(' ');
    }
    for _ in err.start()..err.end() {
        res.push('^');
    }
    res
}
