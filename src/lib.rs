extern crate core;
extern crate rand;

use rand::Rng;

#[derive(Copy, Clone, Debug)]
pub struct Ssn {
    day: u32,
    month: u32,
    year: u32,
    gender: Gender
}

#[derive(Copy, Clone, Debug)]
pub enum Gender {
    Female,
    Male
}

#[derive(Copy, Clone, Debug)]
pub struct Error;

pub fn is_valid(ssn : &str) -> Result<Ssn, Error> {
    if ssn.len() != 11 {
        return Err(Error {});
    }
    let chars: Vec<char> = ssn.chars().collect();

    let separator = chars[6];
    match separator {
        '+' => (),
        '-' => (),
        'A' => (),
        _ => return Err(Error {})
    };

    let date: u32 = match ssn[0..6].parse::<u32>() {
        Ok(n) => n,
        Err(_) => return Err(Error {})
    };

    let month = date % 10000 / 100;
    if month < 1 || month > 12 {
        return Err(Error {});
    }

    let year = date % 100 + match separator {
        '+' => 1800,
        '-' => 1900,
        'A' => 2000,
        _ => return Err(Error {})
    };

    let days_in_month = days_in_month(month, year);
    let day = date / 10000;
    if day < 1 || day > days_in_month {
        return Err(Error {});
    }

    let identifier: u32 = match ssn[7..10].parse::<u32>() {
        Ok(n) => n,
        Err(_) => return Err(Error {})
    };
    if identifier < 002 || identifier > 899 {
        return Err(Error {})
    }

    let checksum = checksum(&ssn);
    if checksum != chars[10] {
        return Err(Error {});
    }

    let gender: Gender = if identifier % 2 == 0 { Gender::Female } else { Gender::Male };

    Ok(Ssn {
        day: day,
        month: month,
        year: year,
        gender: gender
    })
}

pub fn generate() -> String {
    let mut rng = rand::thread_rng();

    let year = rng.gen_range(1890, 2016);
    let month = rng.gen_range(1, 13);
    let day = rng.gen_range(1, days_in_month(month, year) + 1);
    let separator = match year / 100 {
        18 => "+",
        19 => "-",
        20 => "A",
        _ => panic!()
    };
    let identifier = rng.gen_range(200, 900);
    let checksum = CHECKSUM_TABLE[251076155 % 31];
    format!("{:02.}{:02.}{:02.}{}{:03.}{}", day, month, year / 100, separator, identifier, checksum)
}

fn days_in_month(month: u32, year: u32) -> u32 {
    match month {
      1 => 31,
      2 => if is_leap_year(year) { 29 } else { 28 },
      3 => 31,
      4 => 30,
      5 => 31,
      6 => 30,
      7 => 31,
      8 => 31,
      9 => 30,
      10 => 31,
      11 => 30,
      12 => 31,
      _ => panic!()
    }
}

static CHECKSUM_TABLE: [char; 31] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
                 'A', 'B', 'C', 'D', 'E', 'F', 'H', 'J', 'K', 'L',
                 'M', 'N', 'P', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y'];

fn checksum(ssn : &str) -> char {
    let mut hello: String = ssn[..6].to_string();
    hello.push_str(&ssn[7..10]);
    let nums: usize = (&hello).parse().unwrap();

    CHECKSUM_TABLE[nums % 31]
}

fn is_leap_year(year: u32) -> bool {
    ((year % 4 == 0) && (year % 100 != 0)) || (year % 400 == 0)
}

#[test]
fn test_is_valid() {
    assert!(!is_valid("").is_ok(), "Should fail when given empty String");
    assert!(!is_valid("301398-1233").is_ok(), "Should fail when given birthdate with month out of bounds");
    assert!(!is_valid("320198-123P").is_ok(), "Should fail when given birthdate with date out of bounds in January");
    assert!(!is_valid("290299-123U").is_ok(), "Should fail when given birthdate with date out of bounds in February, non leap year");
    assert!(!is_valid("300204-123Y").is_ok(), "Should fail when given birth date with date out of bounds in February, a leap year");
    assert!(!is_valid("0101AA-123A").is_ok(), "Should fail when given birth date with alphabets");
    assert!(!is_valid("010195_433X").is_ok(), "Should fail when given invalid separator chars");
    assert!(!is_valid("01011995+433X").is_ok(), "Should fail when given too long date");
    assert!(!is_valid("01015+433X").is_ok(), "Should fail when given too short date");
    assert!(!is_valid("010195+4433X").is_ok(), "Should fail when given too long checksum part");
    assert!(!is_valid("010195+33X").is_ok(), "Should fail when given too long checksum part");
    assert!(is_valid("010195+433X").is_ok(), "Should pass when given valid finnishSSN from 19th century");
    assert!(is_valid("010197+100P").is_ok(), "Should pass when given valid finnishSSN from 20th century");
    assert!(is_valid("010114A173M").is_ok(), "Should pass when given valid finnishSSN from 21st century");
    assert!(is_valid("290296-7808").is_ok(), "Should pass when given valid finnishSSN with leap year, divisible only by 4");
    assert!(!is_valid("290200-101P").is_ok(), "Should fail when given valid finnishSSN with leap year, divisible by 100 and not by 400");
    assert!(!is_valid("010114A173M ").is_ok(), "Should fail when given SSN longer than 11 chars, bogus in the end");
    assert!(!is_valid(" 010114A173M").is_ok(), "Should fail when given SSN longer than 11 chars, bogus in the beginning");
    assert!(is_valid("290200A248A").is_ok(), "Should pass when given valid finnishSSN with leap year, divisible by 100 and by 400");
}
