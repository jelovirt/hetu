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
            return Err(ParseError::Syntax("Invalid length", 0, ssn.len()));
        }
        let chars: Vec<char> = ssn.chars().collect();

        let separator = chars[6];
        from_separator(separator)?;

        let date: usize = match ssn[0..6].parse::<usize>() {
            Ok(n) => n,
            Err(_) => return Err(ParseError::Syntax("Date not integer", 0, 6)),
        };

        let month = date % 10_000 / 100;
        if !(1..=12).contains(&month) {
            return Err(ParseError::Month("Invalid month number", 2, 4));
        }

        let year = date % 100 + from_separator(separator)?;

        let days_in_month = days_in_month(month, year);
        let day = date / 10_000;
        if day < 1 || day > days_in_month {
            return Err(ParseError::Day("Invalid day number", 0, 2));
        }

        let identifier: usize = match ssn[7..10].parse::<usize>() {
            Ok(n) => n,
            Err(_) => return Err(ParseError::Identifier("Invalid identifier", 7, 10)),
        };
        if !(2..=899).contains(&identifier) {
            return Err(ParseError::Identifier("Invalid identifier number", 10, 11));
        }

        let checksum = checksum(&ssn);
        if checksum != chars[10] {
            return Err(ParseError::Checksum("Incorrect checksum", 10, 11, checksum));
        }

        let gender: Gender = if identifier % 2 == 0 {
            Gender::Female
        } else {
            Gender::Male
        };

        Ok(Ssn {
            day,
            month,
            year,
            gender,
        })
    }

    /// Generate random HETU.
    pub fn generate() -> String {
        let mut rng = rand::thread_rng();

        let year = rng.gen_range(1890, 2016);
        let month = rng.gen_range(1, 13);
        let day = rng.gen_range(1, days_in_month(month, year) + 1);
        let separator = to_separator(year).unwrap();
        let identifier = rng.gen_range(2, 900);
        let nums = day * 10_000_000 + month * 100_000 + (year % 100) * 1_000 + identifier;
        let checksum = CHECKSUM_TABLE[nums % 31];
        format!(
            "{:02.}{:02.}{:02.}{}{:03.}{}",
            day,
            month,
            year % 100,
            separator,
            identifier,
            checksum
        )
    }

    /// Generate HETU with matching fields.
    pub fn generate_by_pattern(pattern: &SsnPattern) -> Result<String, GenerateError> {
        let mut rng = rand::thread_rng();

        let century = match pattern
            .sep
            .map(|c| from_separator(c))
            .unwrap_or_else(|| Ok(1800 + rng.gen_range(0, 3) * 100))
        {
            Ok(v) => v,
            Err(_) => return Err(GenerateError),
        };
        let decade = pattern.y1.unwrap_or_else(|| rng.gen_range(0, 10)) as usize;
        let y2 = pattern.y2.unwrap_or_else(|| rng.gen_range(0, 10)) as usize;
        let year = century + decade * 10 + y2;

        let m1 = pattern
            .m1
            .unwrap_or_else(|| rng.gen_range(if pattern.m2.unwrap_or(1) == 0 { 1 } else { 0 }, 2))
            as usize;
        let m2 = pattern.m2.unwrap_or_else(|| {
            rng.gen_range(if m1 == 0 { 1 } else { 0 }, if m1 == 1 { 3 } else { 10 })
        }) as usize;
        let month = m1 * 10 + m2;

        let days_in_month = days_in_month(month, year);
        let d1 = pattern.d1.unwrap_or_else(|| {
            rng.gen_range(
                if pattern.d2.unwrap_or(1) == 0 { 1 } else { 0 },
                if days_in_month % 10 == 3 { 4 } else { 3 },
            )
        }) as usize;
        let d2 = pattern.d2.unwrap_or_else(|| rng.gen_range(0, 10)) as usize;
        let day = d1 * 10 + d2;

        let (identifier, checksum) = if pattern.check.is_some() {
            let get_numbers = |num: Option<u8>| -> Vec<u8> {
                num.map(|v| vec![v])
                    .unwrap_or_else(|| vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
            };
            let res = || -> Option<(usize, char)> {
                let exp = pattern.check.unwrap();
                for i1 in get_numbers(pattern.i1) {
                    for i2 in get_numbers(pattern.i2) {
                        for i3 in get_numbers(pattern.i3) {
                            let identifier = i1 as usize * 100 + i2 as usize * 10 + i3 as usize;
                            let checksum = checksum_num(day, month, year, identifier);
                            if checksum == exp {
                                return Some((identifier, checksum));
                            }
                        }
                    }
                }
                None
            }();
            match res {
                Some((i, c)) => (i, c),
                None => return Err(GenerateError),
            }
        } else {
            let i1 = pattern.i1.unwrap_or_else(|| rng.gen_range(0, 10)) as usize;
            let i2 = pattern.i2.unwrap_or_else(|| rng.gen_range(0, 10)) as usize;
            let i3 = pattern.i3.unwrap_or_else(|| rng.gen_range(0, 10)) as usize;
            let identifier = i1 * 100 + i2 * 10 + i3;
            let nums = day * 10_000_000 + month * 100_000 + (year % 100) * 1_000 + identifier;
            let checksum = CHECKSUM_TABLE[nums % 31];
            (identifier, checksum)
        };

        Ok(format!(
            "{:02.}{:02.}{:02.}{}{:03.}{}",
            day,
            month,
            year % 100,
            to_separator(century)?,
            identifier,
            checksum
        ))
    }
}

/** Parse separator into century. */
fn from_separator<'a>(separator: char) -> Result<usize, ParseError<'a>> {
    match separator {
        '+' => Ok(1800),
        '-' => Ok(1900),
        'A' => Ok(2000),
        _ => return Err(ParseError::Syntax("Invalid separator", 6, 7)),
    }
}

/** Get separator character for year. */
fn to_separator(year: usize) -> Result<char, GenerateError> {
    match year / 100 {
        18 => Ok('+'),
        19 => Ok('-'),
        20 => Ok('A'),
        _ => Err(GenerateError),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct SsnPattern {
    pub d1: Option<u8>,
    pub d2: Option<u8>,
    pub m1: Option<u8>,
    pub m2: Option<u8>,
    pub y1: Option<u8>,
    pub y2: Option<u8>,
    pub sep: Option<char>,
    pub i1: Option<u8>,
    pub i2: Option<u8>,
    pub i3: Option<u8>,
    pub check: Option<char>,
}

impl SsnPattern {
    pub fn new() -> SsnPattern {
        SsnPattern {
            d1: None,
            d2: None,
            m1: None,
            m2: None,
            y1: None,
            y2: None,
            sep: None,
            i1: None,
            i2: None,
            i3: None,
            check: None,
        }
    }
    fn parse_char(chars: &str, index: usize) -> Result<Option<u8>, ParseError> {
        let c = chars.chars().nth(index).unwrap();
        if c == '?' {
            Ok(None)
        } else {
            Ok(Some(match chars[index..=index].parse::<u8>() {
                Ok(n) => n,
                Err(_) => return Err(ParseError::Syntax("Date not integer", index, index + 1)),
            }))
        }
    }

    pub fn parse(p: &str) -> Result<SsnPattern, ParseError> {
        if p.len() != 11 {
            return Err(ParseError::Syntax("Invalid length", 0, p.len()));
        }
        let d1: Option<u8> = SsnPattern::parse_char(&p, 0)?;
        let d2: Option<u8> = SsnPattern::parse_char(&p, 1)?;
        let m1: Option<u8> = SsnPattern::parse_char(&p, 2)?;
        let m2: Option<u8> = SsnPattern::parse_char(&p, 3)?;
        let y1: Option<u8> = SsnPattern::parse_char(&p, 4)?;
        let y2: Option<u8> = SsnPattern::parse_char(&p, 5)?;
        let sep: Option<char> = match p.chars().nth(6).unwrap() {
            '?' => None,
            sep @ '+' => Some(sep),
            sep @ '-' => Some(sep),
            sep @ 'A' => Some(sep),
            _ => {
                return Err(ParseError::Syntax("Invalid separator character", 6, 7));
            }
        };
        let i1: Option<u8> = SsnPattern::parse_char(&p, 7)?;
        let i2: Option<u8> = SsnPattern::parse_char(&p, 8)?;
        let i3: Option<u8> = SsnPattern::parse_char(&p, 9)?;
        let check: Option<char> = match p.chars().nth(10).unwrap() {
            '?' => None,
            // TODO validate that this is checksum character
            sep => Some(sep),
        };
        Ok(SsnPattern {
            d1,
            d2,
            m1,
            m2,
            y1,
            y2,
            sep,
            i1,
            i2,
            i3,
            check,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gender {
    Female,
    Male,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParseError<'a> {
    Syntax(&'a str, usize, usize),
    Day(&'a str, usize, usize),
    Month(&'a str, usize, usize),
    Year(&'a str, usize, usize),
    Identifier(&'a str, usize, usize),
    Checksum(&'a str, usize, usize, char),
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Syntax(ref desc, _, _) => write!(f, "Invalid syntax: {}", *desc),
            ParseError::Day(ref desc, _, _) => write!(f, "Invalid day: {}", *desc),
            ParseError::Month(ref desc, _, _) => write!(f, "Invalid month: {}", *desc),
            ParseError::Year(ref desc, _, _) => write!(f, "Invalid year: {}", *desc),
            ParseError::Identifier(ref desc, _, _) => write!(f, "Invalid identifier: {}", *desc),
            ParseError::Checksum(_, _, _, ref checksum) => {
                write!(f, "Invalid checksum: expected {}", checksum)
            }
        }
    }
}

impl<'a> error::Error for ParseError<'a> {
    fn description(&self) -> &str {
        match *self {
            ParseError::Syntax(_, _, _) => "Invalid syntax",
            ParseError::Day(_, _, _) => "Invalid day",
            ParseError::Month(_, _, _) => "Invalid month",
            ParseError::Year(_, _, _) => "Invalid year",
            ParseError::Identifier(_, _, _) => "Invalid identifier",
            ParseError::Checksum(_, _, _, _) => "Invalid checksum",
        }
    }

    fn cause(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GenerateError;

impl<'a> fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to generate matching hetu")
    }
}

impl<'a> error::Error for GenerateError {
    fn description(&self) -> &str {
        "Unable to generate matching hetu"
    }

    fn cause(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

pub trait ErrorIndexRange {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

impl<'a> ErrorIndexRange for ParseError<'a> {
    fn start(&self) -> usize {
        match *self {
            ParseError::Syntax(_, start, _) => start,
            ParseError::Day(_, start, _) => start,
            ParseError::Month(_, start, _) => start,
            ParseError::Year(_, start, _) => start,
            ParseError::Identifier(_, start, _) => start,
            ParseError::Checksum(_, start, _, _) => start,
        }
    }
    fn end(&self) -> usize {
        match *self {
            ParseError::Syntax(_, _, end) => end,
            ParseError::Day(_, _, end) => end,
            ParseError::Month(_, _, end) => end,
            ParseError::Year(_, _, end) => end,
            ParseError::Identifier(_, _, end) => end,
            ParseError::Checksum(_, _, end, _) => end,
        }
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

static CHECKSUM_TABLE: [char; 31] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'H', 'J', 'K',
    'L', 'M', 'N', 'P', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y',
];

fn checksum(ssn: &str) -> char {
    let mut hello: String = ssn[..6].to_string();
    hello.push_str(&ssn[7..10]);
    let nums: usize = (&hello).parse().unwrap();
    CHECKSUM_TABLE[nums % 31]
}
fn checksum_num(day: usize, month: usize, year: usize, identifier: usize) -> char {
    let nums = day * 10_000_000 + month * 100_000 + (year % 100) * 1_000 + identifier;
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
        assert!(
            Ssn::parse("").unwrap_err() == ParseError::Syntax("Invalid length", 0, 0),
            "fail when given empty String"
        );
        assert!(
            Ssn::parse("301398-1233").unwrap_err()
                == ParseError::Month("Invalid month number", 2, 4),
            "fail when given birthdate with month out of bounds"
        );
        assert!(
            Ssn::parse("320198-123P").unwrap_err() == ParseError::Day("Invalid day number", 0, 2),
            "fail when given birthdate with date out of bounds in January"
        );
        assert!(
            Ssn::parse("290299-123U").unwrap_err() == ParseError::Day("Invalid day number", 0, 2),
            "fail when given birthdate with date out of bounds in February, non leap year"
        );
        assert!(
            Ssn::parse("300204-123Y").unwrap_err() == ParseError::Day("Invalid day number", 0, 2),
            "fail when given birth date with date out of bounds in February, a leap year"
        );
        assert!(
            Ssn::parse("0101AA-123A").unwrap_err() == ParseError::Syntax("Date not integer", 0, 6),
            "fail when given birth date with alphabets"
        );
        assert!(
            Ssn::parse("010195_433X").unwrap_err() == ParseError::Syntax("Invalid separator", 6, 7),
            "fail when given invalid separator chars"
        );
        assert!(
            Ssn::parse("01011995+433X").unwrap_err() == ParseError::Syntax("Invalid length", 0, 13),
            "fail when given too long date"
        );
        assert!(
            Ssn::parse("01015+433X").unwrap_err() == ParseError::Syntax("Invalid length", 0, 10),
            "fail when given too short date"
        );
        assert!(
            Ssn::parse("010195+4433X").unwrap_err() == ParseError::Syntax("Invalid length", 0, 12),
            "fail when given too long checksum part"
        );
        assert!(
            Ssn::parse("010195+33X").unwrap_err() == ParseError::Syntax("Invalid length", 0, 10),
            "fail when given too long checksum part"
        );
        assert_eq!(
            Ssn::parse("010195+433X").unwrap(),
            Ssn {
                day: 1,
                month: 1,
                year: 1895,
                gender: Gender::Male,
            }
        );
        assert_eq!(
            Ssn::parse("010197-100P").unwrap(),
            Ssn {
                day: 1,
                month: 1,
                year: 1997,
                gender: Gender::Female,
            }
        );
        assert_eq!(
            Ssn::parse("010114A173M").unwrap(),
            Ssn {
                day: 1,
                month: 1,
                year: 2014,
                gender: Gender::Male,
            }
        );
        // pass when given valid finnishSSN with leap year, divisible only by 4
        assert_eq!(
            Ssn::parse("290296-7808").unwrap(),
            Ssn {
                day: 29,
                month: 2,
                year: 1996,
                gender: Gender::Female,
            }
        );
        assert!(
            Ssn::parse("290200-101P").unwrap_err() == ParseError::Day("Invalid day number", 0, 2),
            "fail when given valid finnishSSN with leap year, divisible by 100 and not by 400"
        );
        // pass when given valid finnishSSN with leap year, divisible by 100 and by 400
        assert_eq!(
            Ssn::parse("290200A248A").unwrap(),
            Ssn {
                day: 29,
                month: 2,
                year: 2000,
                gender: Gender::Female,
            }
        );
        assert!(
            Ssn::parse("010114A173M ").unwrap_err() == ParseError::Syntax("Invalid length", 0, 12),
            "fail when given SSN longer than 11 chars, bogus in the end"
        );
        assert!(
            Ssn::parse(" 010114A173M").unwrap_err() == ParseError::Syntax("Invalid length", 0, 12),
            "fail when given SSN longer than 11 chars, bogus in the beginning"
        );
    }

    #[test]
    fn test_generate() {
        let ssn = Ssn::generate();
        assert!(Ssn::parse(&ssn).is_ok());
    }

    #[test]
    fn test_pattern_parse() {
        assert!(
            SsnPattern::parse("").unwrap_err() == ParseError::Syntax("Invalid length", 0, 0),
            "fail when given empty String"
        );
        assert!(
            SsnPattern::parse("123456X7890").unwrap_err()
                == ParseError::Syntax("Invalid separator character", 6, 7),
            "fail when separator is not valid character"
        );
        assert!(SsnPattern::parse("123456-7890").is_ok(), "parse valid SSN");
        assert!(
            SsnPattern::parse("??????-????").is_ok(),
            "parse all wildcard input"
        );
    }

    #[test]
    fn test_generate_pattern() {
        assert!(
            Ssn::generate_by_pattern(&SsnPattern::parse("111111-111?").unwrap()).unwrap()
                == "111111-111C",
            "generate expected SSN"
        );
    }
}
