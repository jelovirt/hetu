extern crate core;
extern crate rand;

use rand::Rng;
use std::error;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ssn {
    pub day: usize,
    pub month: usize,
    pub year: usize,
    pub gender: Gender,
}

impl Ssn {
    /// Parse HETU.
    pub fn parse(ssn: &str) -> Result<Ssn, ParseError> {
        if ssn.len() != 11 {
            return Err(ParseError::Syntax("Invalid length"));
        }
        let chars: Vec<char> = ssn.chars().collect();

        let separator = chars[6];
        match separator {
            '+' => (),
            '-' => (),
            'A' => (),
            _ => return Err(ParseError::Syntax("Invalid separator")),
        };

        let date: usize = match ssn[0..6].parse::<usize>() {
            Ok(n) => n,
            Err(_) => return Err(ParseError::Syntax("Date not integer")),
        };

        let month = date % 10000 / 100;
        if month < 1 || month > 12 {
            return Err(ParseError::Month("Invalid month number"));
        }

        let year = date % 100 +
                   match separator {
            '+' => 1800,
            '-' => 1900,
            'A' => 2000,
            _ => return Err(ParseError::Syntax("Invalid separator")),
        };

        let days_in_month = days_in_month(month, year);
        let day = date / 10000;
        if day < 1 || day > days_in_month {
            return Err(ParseError::Day("Invalid day number"));
        }

        let identifier: usize = match ssn[7..10].parse::<usize>() {
            Ok(n) => n,
            Err(_) => return Err(ParseError::Identifier("Invalid identifier")),
        };
        if identifier < 002 || identifier > 899 {
            return Err(ParseError::Identifier("Invalid identifier number"));
        }

        let checksum = checksum(&ssn);
        if checksum != chars[10] {
            return Err(ParseError::Checksum("Incorrect checksum"));
        }

        let gender: Gender = if identifier % 2 == 0 {
            Gender::Female
        } else {
            Gender::Male
        };

        Ok(Ssn {
            day: day,
            month: month,
            year: year,
            gender: gender,
        })
    }

    /// Generate random HETU.
    pub fn generate() -> String {
        let mut rng = rand::thread_rng();

        let year = rng.gen_range(1890, 2016);
        let month = rng.gen_range(1, 13);
        let day = rng.gen_range(1, days_in_month(month, year) + 1);
        let separator = match year / 100 {
            18 => "+",
            19 => "-",
            20 => "A",
            _ => panic!(),
        };
        let identifier = rng.gen_range(002, 900);
        let nums = day * 10000000 + month * 100000 + (year % 100) * 1000 + identifier;
        let checksum = CHECKSUM_TABLE[nums % 31];
        format!("{:02.}{:02.}{:02.}{}{:03.}{}",
                day,
                month,
                year % 100,
                separator,
                identifier,
                checksum)
    }

    /// Generate HETU with matching fields.
    pub fn generate_by_pattern(ssn: &Ssn) -> String {
        let mut rng = rand::thread_rng();

        let separator = match ssn.year / 100 {
            18 => "+",
            19 => "-",
            20 => "A",
            _ => panic!(),
        };
        let identifier = (rng.gen_range(002, 900) / 2) * 2 +
                         (if ssn.gender == Gender::Male {
            1
        } else {
            0
        });
        let nums = ssn.day * 10000000 + ssn.month * 100000 + (ssn.year % 100) * 1000 + identifier;
        let checksum = CHECKSUM_TABLE[nums % 31];
        format!("{:02.}{:02.}{:02.}{}{:03.}{}",
                ssn.day,
                ssn.month,
                ssn.year % 100,
                separator,
                identifier,
                checksum)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gender {
    Female,
    Male,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParseError<'a> {
    Syntax(&'a str),
    Day(&'a str),
    Month(&'a str),
    Year(&'a str),
    Identifier(&'a str),
    Checksum(&'a str),
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Syntax(ref desc) => write!(f, "Invalid syntax: {}", *desc),
            ParseError::Day(ref desc) => write!(f, "Invalid day: {}", desc),
            ParseError::Month(ref desc) => write!(f, "Invalid month: {}", *desc),
            ParseError::Year(ref desc) => write!(f, "Invalid year: {}", *desc),
            ParseError::Identifier(ref desc) => write!(f, "Invalid identifier: {}", *desc),
            ParseError::Checksum(ref desc) => write!(f, "Invalid checksum: {}", *desc),
        }
    }
}

impl<'a> error::Error for ParseError<'a> {
    fn description(& self) -> &str {
        match *self {
            ParseError::Syntax(_) => "Invalid syntax",
            ParseError::Day(_) => "Invalid day",
            ParseError::Month(_) => "Invalid month",
            ParseError::Year(_) => "Invalid year",
            ParseError::Identifier(_) => "Invalid identifier",
            ParseError::Checksum(_) => "Invalid checksum",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

fn days_in_month(month: usize, year: usize) -> usize {
    match month {
        1 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
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
        _ => panic!(),
    }
}

static CHECKSUM_TABLE: [char; 31] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B',
                                     'C', 'D', 'E', 'F', 'H', 'J', 'K', 'L', 'M', 'N', 'P', 'R',
                                     'S', 'T', 'U', 'V', 'W', 'X', 'Y'];

fn checksum(ssn: &str) -> char {
    let mut hello: String = ssn[..6].to_string();
    hello.push_str(&ssn[7..10]);
    let nums: usize = (&hello).parse().unwrap();

    CHECKSUM_TABLE[nums % 31]
}

fn is_leap_year(year: usize) -> bool {
    ((year % 4 == 0) && (year % 100 != 0)) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        assert!(Ssn::parse("").unwrap_err() == ParseError::Syntax("Invalid length"),
                "fail when given empty String");
        assert!(Ssn::parse("301398-1233").unwrap_err() == ParseError::Month("Invalid month number"),
        "fail when given birthdate with month out of bounds");
        assert!(Ssn::parse("320198-123P").unwrap_err() == ParseError::Day("Invalid day number"),
        "fail when given birthdate with date out of bounds in January");
        assert!(Ssn::parse("290299-123U").unwrap_err() == ParseError::Day("Invalid day number"),
        "fail when given birthdate with date out of bounds in February, non leap year");
        assert!(Ssn::parse("300204-123Y").unwrap_err() == ParseError::Day("Invalid day number"),
        "fail when given birth date with date out of bounds in February, a leap year");
        assert!(Ssn::parse("0101AA-123A").unwrap_err() == ParseError::Syntax("Date not integer"),
                "fail when given birth date with alphabets");
        assert!(Ssn::parse("010195_433X").unwrap_err() == ParseError::Syntax("Invalid separator"),
                "fail when given invalid separator chars");
        assert!(Ssn::parse("01011995+433X").unwrap_err() == ParseError::Syntax("Invalid length"),
                "fail when given too long date");
        assert!(Ssn::parse("01015+433X").unwrap_err() == ParseError::Syntax("Invalid length"),
                "fail when given too short date");
        assert!(Ssn::parse("010195+4433X").unwrap_err() == ParseError::Syntax("Invalid length"),
                "fail when given too long checksum part");
        assert!(Ssn::parse("010195+33X").unwrap_err() == ParseError::Syntax("Invalid length"),
                "fail when given too long checksum part");
        assert_eq!(Ssn::parse("010195+433X").unwrap(),
                   Ssn {
                       day: 1,
                       month: 1,
                       year: 1895,
                       gender: Gender::Male,
                   });
        assert_eq!(Ssn::parse("010197-100P").unwrap(),
                   Ssn {
                       day: 1,
                       month: 1,
                       year: 1997,
                       gender: Gender::Female,
                   });
        assert_eq!(Ssn::parse("010114A173M").unwrap(),
                   Ssn {
                       day: 1,
                       month: 1,
                       year: 2014,
                       gender: Gender::Male,
                   });
        // pass when given valid finnishSSN with leap year, divisible only by 4
        assert_eq!(Ssn::parse("290296-7808").unwrap(),
                   Ssn {
                       day: 29,
                       month: 2,
                       year: 1996,
                       gender: Gender::Female,
                   });
        assert!(Ssn::parse("290200-101P").unwrap_err() == ParseError::Day("Invalid day number"),
                "fail when given valid finnishSSN with leap year, divisible by 100 and not by 400");
        // pass when given valid finnishSSN with leap year, divisible by 100 and by 400
        assert_eq!(Ssn::parse("290200A248A").unwrap(),
                   Ssn {
                       day: 29,
                       month: 2,
                       year: 2000,
                       gender: Gender::Female,
                   });
        assert!(Ssn::parse("010114A173M ").unwrap_err() == ParseError::Syntax("Invalid length"),
                "fail when given SSN longer than 11 chars, bogus in the end");
        assert!(Ssn::parse(" 010114A173M").unwrap_err() == ParseError::Syntax("Invalid length"),
                "fail when given SSN longer than 11 chars, bogus in the beginning");
    }

    #[test]
    fn test_generate() {
        let ssn = Ssn::generate();
        assert!(Ssn::parse(&ssn).is_ok());
    }
}
