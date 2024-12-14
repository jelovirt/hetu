extern crate core;
extern crate rand;

use rand::{Rng, ThreadRng};
use std::error;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ssn {
    pub day: usize,
    pub month: usize,
    pub year: usize,
    pub gender: Gender,
}

fn century_range(sep: &Option<char>) -> Vec<usize> {
    match sep {
        Some(c) => match from_separator(c) {
            Ok(v) => vec![v],
            Err(_) => panic!("Unsupported separator in pattern"),
        },
        None => vec![1800usize, 1900usize, 2000usize],
    }
}

static SEPARATORS: [char; 13] = [
    '+', '-', 'Y', 'X', 'W', 'V', 'U', 'A', 'B', 'C', 'D', 'E', 'F',
];

fn separator_range(sep: &Option<char>) -> Vec<char> {
    match sep {
        Some(c) => vec![*c],
        None => SEPARATORS.to_vec(),
    }
}

fn decade_range(y1: &Option<u8>, sep: &Option<char>) -> Vec<usize> {
    match (y1, sep) {
        (Some(v), _) => vec![*v as usize],
        (None, Some('+')) => (5usize..=9usize).collect(),
        (None, _) => (0usize..=9usize).collect(),
    }
}

fn y2_range(rng: &mut ThreadRng, y2: &Option<u8>) -> Vec<usize> {
    match y2 {
        Some(v) => vec![*v as usize],
        None => {
            let mut range: Vec<usize> = (0usize..=9usize).collect();
            rng.shuffle(&mut range);
            range
        }
    }
}

fn m_range(m1: &Option<u8>, m2: &Option<u8>) -> Vec<usize> {
    match (m1, m2) {
        (Some(ref m1), Some(ref m2)) => {
            let m = (m1 * 10 + m2) as usize;
            if !(1..=12).contains(&m) {
                panic!("Unsupported month {} in pattern", &m);
            };
            vec![m]
        }
        (Some(ref m1), None) => ((if *m1 == 0 { 1 } else { 0 })..(if *m1 == 0 { 10 } else { 3 }))
            .map(|m2| (m1 * 10 + m2) as usize)
            .collect(),
        (None, Some(ref m2)) => ((if *m2 == 0 { 1 } else { 0 })..2)
            .map(|m1| (m1 * 10 + m2) as usize)
            .collect(),
        (None, None) => (1usize..13usize).collect(),
    }
}

fn d_range(d1: &Option<u8>, d2: &Option<u8>) -> Vec<usize> {
    match (d1, d2) {
        (Some(ref d1), Some(ref d2)) => {
            let d = (d1 * 10 + d2) as usize;
            if d < 1 {
                panic!("Unsupported day {} in pattern", &d);
            };
            vec![d]
        }
        (Some(ref d1), None) => {
            // if *d1 < 1 || *d1 as usize > days_in_month / 10 {
            //     return Err(GenerateError);
            // };
            ((if *d1 as usize == 0 { 1 } else { 0 })..=9)
                .map(|d2| *d1 as usize * 10 + d2)
                .collect()
        }
        (None, Some(ref d2)) => ((if *d2 as usize == 0 { 1 } else { 0 })..4)
            .map(|d1| d1 * 10 + *d2 as usize)
            .collect(),
        (None, None) =>
        // (1..(days_in_month + 1)).collect()
        {
            (1..=31).collect()
        }
    }
}

pub fn generate_by_pattern_with_any_checksum(
    pattern: &SsnPattern,
) -> Result<String, GenerateError> {
    let mut rng = rand::thread_rng();

    let century = match pattern
        .sep
        .map(|c| from_separator(&c))
        .unwrap_or_else(|| Ok(1800 + rng.gen_range(0, 3) * 100))
    {
        Ok(v) => v,
        Err(_) => return Err(GenerateError),
    };
    // let separator: char = to_separator(century, &mut rng)?;
    let separator: char = match pattern.sep {
        Some(s) => s,
        None => match century {
            1800 => '+',
            1900 => *rng.choose(&['-', 'Y', 'X', 'W', 'V', 'U']).unwrap(),
            2000 => *rng.choose(&['A', 'B', 'C', 'D', 'E', 'F']).unwrap(),
            _ => return Err(GenerateError),
        },
    };
    // Unless the pattern explicitly sets the year to be before 1850, don't generate years before 1850.
    let decade = pattern
        .y1
        .unwrap_or_else(|| rng.gen_range(if century == 1800 { 5 } else { 0 }, 10))
        as usize;
    let y2 = pattern.y2.unwrap_or_else(|| rng.gen_range(0, 10)) as usize;
    let year = century + decade * 10 + y2;

    let month: usize = match (pattern.m1, pattern.m2) {
        (Some(ref m1), Some(ref m2)) => {
            let m = (m1 * 10 + m2) as usize;
            if !(1..=12).contains(&m) {
                return Err(GenerateError);
            };
            m
        }
        (Some(ref m1), None) => {
            let m2 = rng.gen_range(if *m1 == 0 { 1 } else { 0 }, if *m1 == 0 { 10 } else { 3 });
            (m1 * 10 + m2) as usize
        }
        (None, Some(ref m2)) => {
            let m1 = rng.gen_range(if *m2 == 0 { 1 } else { 0 }, 2);
            (m1 * 10 + m2) as usize
        }
        (None, None) => rng.gen_range(1, 13),
    };

    let days_in_month = days_in_month(month, year);
    let day: usize = match (pattern.d1, pattern.d2) {
        (Some(ref d1), Some(ref d2)) => {
            let d = (d1 * 10 + d2) as usize;
            if d < 1 || d > days_in_month {
                return Err(GenerateError);
            };
            d
        }
        (Some(ref d1), None) => {
            if *d1 as usize > days_in_month / 10 {
                return Err(GenerateError);
            };
            let d2 = rng.gen_range(if *d1 as usize == 0 { 1 } else { 0 }, 10);
            *d1 as usize * 10 + d2
        }
        (None, Some(ref d2)) => {
            let d1 = rng.gen_range(
                if *d2 as usize == 0 { 1 } else { 0 },
                if days_in_month % 10 == 3 { 4 } else { 3 },
            ) as usize;
            d1 * 10 + *d2 as usize
        }
        (None, None) => rng.gen_range(1, days_in_month + 1),
    };

    let i1 = pattern.i1.unwrap_or_else(|| rng.gen_range(0, 9)) as usize;
    let i2 = pattern.i2.unwrap_or_else(|| rng.gen_range(0, 10)) as usize;
    let i3 = pattern
        .i3
        .unwrap_or_else(|| rng.gen_range(if i1 == 0 && i2 == 0 { 2 } else { 0 }, 10))
        as usize;
    let identifier = i1 * 100 + i2 * 10 + i3;
    if identifier < 2 {
        return Err(GenerateError);
    }
    let nums = day * 10_000_000 + month * 100_000 + (year % 100) * 1_000 + identifier;
    let checksum = CHECKSUM_TABLE[nums % 31];

    Ok(format!(
        "{:02}{:02}{:02}{}{:03}{}",
        day,
        month,
        year % 100,
        separator,
        identifier,
        checksum
    ))
}

pub fn generate_by_pattern_with_fixed_checksum(
    pattern: &SsnPattern,
) -> Result<String, GenerateError> {
    let mut rng = rand::thread_rng();
    let mut centuries = century_range(&pattern.sep);
    rng.shuffle(&mut centuries);
    let mut separators = separator_range(&pattern.sep);
    rng.shuffle(&mut separators);
    let mut decades = decade_range(&pattern.y1, &pattern.sep);
    rng.shuffle(&mut decades);
    let mut y2s = y2_range(&mut rng, &pattern.y2);
    rng.shuffle(&mut y2s);
    let mut months = m_range(&pattern.m1, &pattern.m2);
    rng.shuffle(&mut months);
    let mut days = d_range(&pattern.d1, &pattern.d2);
    rng.shuffle(&mut days);
    let mut i1s = pattern
        .i1
        .map(|v| vec![v as usize])
        .unwrap_or_else(|| (0usize..=8usize).collect());
    rng.shuffle(&mut i1s);
    let mut i2s = pattern
        .i2
        .map(|v| vec![v as usize])
        .unwrap_or_else(|| (0usize..=9usize).collect());
    rng.shuffle(&mut i2s);
    let mut i3s = pattern
        .i3
        .map(|v| vec![v as usize])
        // .unwrap_or_else(|| ((if i1 == 0 && i2 == 0 { 2usize } else { 0usize })..10usize)
        .unwrap_or_else(|| (0usize..=9usize).collect());
    rng.shuffle(&mut i3s);
    for century in &centuries {
        for separator in &separators {
            for decade in &decades {
                for y2 in &y2s {
                    let year = century + decade * 10 + y2;
                    if year < 1850 {
                        continue;
                    }
                    for month in &months {
                        let days_in_this_month = days_in_month(*month, year);
                        for day in days.iter().filter(|d| d <= &&days_in_this_month) {
                            for i1 in &i1s {
                                for i2 in &i2s {
                                    for i3 in &i3s {
                                        let identifier = i1 * 100 + i2 * 10 + i3;
                                        if identifier < 2 {
                                            continue;
                                        }
                                        let nums = day * 10_000_000
                                            + month * 100_000
                                            + (year % 100) * 1_000
                                            + identifier;
                                        let exp_checksum = CHECKSUM_TABLE[nums % 31];
                                        let checksum = &pattern.check.unwrap();
                                        if exp_checksum != *checksum {
                                            // println!("{} != {} was not valid guess", exp_checksum, checksum);
                                            continue;
                                        }
                                        return Ok(format!(
                                            "{:02}{:02}{:02}{}{:03}{}",
                                            day,
                                            month,
                                            year % 100,
                                            separator,
                                            identifier,
                                            checksum
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Err(GenerateError)
}

#[derive(Debug)]
struct SsnIterator {
    all: Vec<usize>,
    bases: [usize; 8],
    offsets: [usize; 8],
    separators: Vec<char>,
    check: Option<char>,
    offset: usize,
}

impl SsnIterator {
    const OFFSET_CENTURY: usize = 0;
    const OFFSET_DECADE: usize = 1;
    const OFFSET_Y2: usize = 2;
    const OFFSET_MOTH: usize = 3;
    const OFFSET_DAY: usize = 4;
    const OFFSET_I1: usize = 5;
    const OFFSET_I2: usize = 6;
    const OFFSET_I3: usize = 7;

    pub fn new(pattern: &SsnPattern) -> SsnIterator {
        let mut rng = rand::thread_rng();

        let mut centuries = century_range(&pattern.sep);
        rng.shuffle(&mut centuries);
        let mut separators = separator_range(&pattern.sep);
        rng.shuffle(&mut separators);
        let mut decades = decade_range(&pattern.y1, &pattern.sep);
        rng.shuffle(&mut decades);
        let mut y2s = y2_range(&mut rng, &pattern.y2);
        rng.shuffle(&mut y2s);
        let mut months = m_range(&pattern.m1, &pattern.m2);
        rng.shuffle(&mut months);
        let mut days = d_range(&pattern.d1, &pattern.d2);
        rng.shuffle(&mut days);
        let mut i1s = pattern
            .i1
            .map(|v| vec![v as usize])
            .unwrap_or_else(|| (0usize..=8usize).collect());
        rng.shuffle(&mut i1s);
        let mut i2s = pattern
            .i2
            .map(|v| vec![v as usize])
            .unwrap_or_else(|| (0usize..=9usize).collect());
        rng.shuffle(&mut i2s);
        let mut i3s = pattern
            .i3
            .map(|v| vec![v as usize])
            .unwrap_or_else(|| (0usize..=9usize).collect());
        rng.shuffle(&mut i3s);

        let all: Vec<usize> = vec![];

        let mut res = SsnIterator {
            all,
            bases: [
                centuries.len(),
                centuries.len() * decades.len(),
                centuries.len() * decades.len() * y2s.len(),
                centuries.len() * decades.len() * y2s.len() * months.len(),
                centuries.len() * decades.len() * y2s.len() * months.len() * days.len(),
                centuries.len() * decades.len() * y2s.len() * months.len() * days.len() * i1s.len(),
                centuries.len()
                    * decades.len()
                    * y2s.len()
                    * months.len()
                    * days.len()
                    * i1s.len()
                    * i2s.len(),
                centuries.len()
                    * decades.len()
                    * y2s.len()
                    * months.len()
                    * days.len()
                    * i1s.len()
                    * i2s.len()
                    * i3s.len(),
            ],
            offsets: [
                0,
                centuries.len(),
                centuries.len() + decades.len(),
                centuries.len() + decades.len() + y2s.len(),
                centuries.len() + decades.len() + y2s.len() + months.len(),
                centuries.len() + decades.len() + y2s.len() + months.len() + days.len(),
                centuries.len() + decades.len() + y2s.len() + months.len() + days.len() + i1s.len(),
                centuries.len()
                    + decades.len()
                    + y2s.len()
                    + months.len()
                    + days.len()
                    + i1s.len()
                    + i2s.len(),
            ],
            separators,
            check: pattern.check,
            offset: usize::MAX,
        };

        res.all.append(&mut centuries);
        res.all.append(&mut decades);
        res.all.append(&mut y2s);
        res.all.append(&mut months);
        res.all.append(&mut days);
        res.all.append(&mut i1s);
        res.all.append(&mut i2s);
        res.all.append(&mut i3s);

        res
    }
}

impl Iterator for SsnIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.offset == usize::MAX {
                self.offset = 0;
            } else {
                self.offset += 1;
            }

            // year
            let century_index = (self.offset % self.bases[SsnIterator::OFFSET_CENTURY])
                + self.offsets[SsnIterator::OFFSET_CENTURY];
            let century = self.all[century_index];
            let decade_index = (self.offset % self.bases[SsnIterator::OFFSET_DECADE])
                / self.bases[SsnIterator::OFFSET_CENTURY]
                + self.offsets[SsnIterator::OFFSET_DECADE];
            let decade = self.all[decade_index];
            let y2_index = (self.offset % self.bases[SsnIterator::OFFSET_Y2])
                / self.bases[SsnIterator::OFFSET_DECADE]
                + self.offsets[SsnIterator::OFFSET_Y2];
            let y2 = self.all[y2_index];
            // month
            let month_index = (self.offset % self.bases[SsnIterator::OFFSET_MOTH])
                / self.bases[SsnIterator::OFFSET_Y2]
                + self.offsets[SsnIterator::OFFSET_MOTH];
            let month = self.all[month_index];
            // day
            let day_index = (self.offset % self.bases[SsnIterator::OFFSET_DAY])
                / self.bases[SsnIterator::OFFSET_MOTH]
                + self.offsets[SsnIterator::OFFSET_DAY];
            let day = self.all[day_index];
            let year = century + decade * 10 + y2;
            let separator = if self.separators.len() == 1 {
                self.separators.first().unwrap()
            } else {
                self.separators
                    .get(self.offset % self.separators.len())
                    .unwrap()
            };
            let days_in_this_month = days_in_month(month, year);
            if day >= days_in_this_month {
                println!("day was too large");
                continue;
            }
            // identifier
            let i1_index = (self.offset % self.bases[SsnIterator::OFFSET_I1])
                / self.bases[SsnIterator::OFFSET_DAY]
                + self.offsets[SsnIterator::OFFSET_I1];
            let i1 = self.all[i1_index];
            let i2_index = (self.offset % self.bases[SsnIterator::OFFSET_I2])
                / self.bases[SsnIterator::OFFSET_I1]
                + self.offsets[SsnIterator::OFFSET_I2];
            let i2 = self.all[i2_index];
            let i3_index = (self.offset % self.bases[SsnIterator::OFFSET_I3])
                / self.bases[SsnIterator::OFFSET_I2]
                + self.offsets[SsnIterator::OFFSET_I3];
            let i3 = self.all[i3_index];
            let identifier = i1 * 100 + i2 * 10 + i3;
            if identifier < 2 {
                println!("identifier was too small");
                continue;
            }
            // checksum
            let nums = day * 10_000_000 + month * 100_000 + (year % 100) * 1_000 + identifier;
            let exp_checksum = CHECKSUM_TABLE[nums % 31];
            let checksum = match self.check {
                Some(c) => {
                    if exp_checksum != c {
                        println!("{} != {} was not valid guess", exp_checksum, c);
                        continue;
                    }
                    c
                }
                None => exp_checksum,
            };
            return Some(format!(
                "{:02}{:02}{:02}{}{:03}{}",
                day,
                month,
                year % 100,
                separator,
                identifier,
                checksum
            ));
        }
        // None
    }
}

impl Ssn {
    /// Parse HETU.
    pub fn parse(ssn: &str) -> Result<Ssn, ParseError> {
        if ssn.len() != 11 {
            return Err(ParseError::Syntax("Invalid length", 0, ssn.len()));
        }
        let chars: Vec<char> = ssn.chars().collect();

        let separator = chars[6];
        from_separator(&separator)?;

        let date: usize = match ssn[0..6].parse::<usize>() {
            Ok(n) => n,
            Err(_) => return Err(ParseError::Syntax("Date not integer", 0, 6)),
        };

        let month = date % 10_000 / 100;
        if !(1..=12).contains(&month) {
            return Err(ParseError::Month("Invalid month number", 2, 4));
        }

        let year = date % 100 + from_separator(&separator)?;
        if year < 1850 {
            return Err(ParseError::Day("Invalid year before 1850", 4, 6));
        }

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

        let checksum = checksum(ssn);
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
    ///
    /// Temporary HETU with identifier range of 900-999 will never be created. To generate a
    /// temporary HETU, use `Ssn::generate_by_pattern(pattern)` with pattern that explicity has '9' as the
    /// first character of the identifier part.
    pub fn generate() -> String {
        let mut rng = rand::thread_rng();

        let year = rng.gen_range(1890, 2016);
        let month = rng.gen_range(1, 13);
        let day = rng.gen_range(1, days_in_month(month, year) + 1);
        let separator = to_separator(year, &mut rng).unwrap();
        let identifier = rng.gen_range(2, 900);
        let nums = day * 10_000_000 + month * 100_000 + (year % 100) * 1_000 + identifier;
        let checksum = CHECKSUM_TABLE[nums % 31];
        format!(
            "{:02}{:02}{:02}{}{:03}{}",
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
        match &pattern.check {
            Some(_) => generate_by_pattern_with_fixed_checksum(pattern),
            None => generate_by_pattern_with_any_checksum(pattern),
        }
    }

    /// Iterator for generated HETUs with matching fields.
    pub fn iter<'a>(pattern: &SsnPattern) -> impl Iterator<Item = String> + 'a {
        SsnIterator::new(pattern)
    }
}

/** Parse separator into century. */
fn from_separator<'a>(separator: &char) -> Result<usize, ParseError<'a>> {
    match separator {
        '+' => Ok(1800),
        '-' | 'Y' | 'X' | 'W' | 'V' | 'U' => Ok(1900),
        'A' | 'B' | 'C' | 'D' | 'E' | 'F' => Ok(2000),
        _ => Err(ParseError::Syntax("Invalid separator", 6, 7)),
    }
}

/** Get separator character for year. */
fn to_separator(year: usize, rng: &mut ThreadRng) -> Result<char, GenerateError> {
    match year / 100 {
        18 => Ok('+'),
        19 => rng
            .choose(&['-', 'Y', 'X', 'W', 'V', 'U'])
            .ok_or(GenerateError)
            .copied(),
        20 => rng
            .choose(&['A', 'B', 'C', 'D', 'E', 'F'])
            .ok_or(GenerateError)
            .copied(),
        _ => Err(GenerateError),
    }
}

/// Pattern that defines generated Ssn.
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

    /// Parse pattern from a string.
    ///
    /// A character in the pattern string is either the desired character or a wildcard denoted by
    /// a '?' character.
    ///
    /// # Example
    ///
    /// ```
    /// use hetu::SsnPattern;
    ///
    /// // separator character should be '-' to denote birthday between 1900 and 1999
    /// SsnPattern::parse("??????-????");
    /// // all other characters are fixed except the checksum
    /// SsnPattern::parse("141286-245?");
    /// ```
    pub fn parse(p: &str) -> Result<SsnPattern, ParseError> {
        if p.len() != 11 {
            return Err(ParseError::Syntax("Invalid length", 0, p.len()));
        }
        let d1: Option<u8> = SsnPattern::parse_char(p, 0)?;
        let d2: Option<u8> = SsnPattern::parse_char(p, 1)?;
        let m1: Option<u8> = SsnPattern::parse_char(p, 2)?;
        let m2: Option<u8> = SsnPattern::parse_char(p, 3)?;
        let y1: Option<u8> = SsnPattern::parse_char(p, 4)?;
        let y2: Option<u8> = SsnPattern::parse_char(p, 5)?;
        let sep: Option<char> = match p.chars().nth(6).unwrap() {
            '?' => None,
            sep @ '+' => Some(sep),
            sep @ '-' | sep @ 'Y' | sep @ 'X' | sep @ 'W' | sep @ 'V' | sep @ 'U' => Some(sep),
            sep @ 'A' | sep @ 'B' | sep @ 'C' | sep @ 'D' | sep @ 'E' | sep @ 'F' => Some(sep),
            _ => {
                return Err(ParseError::Syntax("Invalid separator character", 6, 7));
            }
        };
        let i1: Option<u8> = SsnPattern::parse_char(p, 7)?;
        let i2: Option<u8> = SsnPattern::parse_char(p, 8)?;
        let i3: Option<u8> = SsnPattern::parse_char(p, 9)?;
        let check: Option<char> = match p.chars().nth(10).unwrap() {
            '?' => None,
            sep if CHECKSUM_TABLE.contains(&sep) => Some(sep),
            _ => {
                return Err(ParseError::Syntax("Invalid checksum character", 10, 11));
            }
        };

        match (d1, d2, m1, m2, y1, sep) {
            (Some(0), Some(0), _, _, _, _) => {
                return Err(ParseError::Day("Invalid day too small", 0, 1));
            }
            (Some(d1), Some(d2), _, _, _, _) if (d1 * 10 + d2) > 31 => {
                return Err(ParseError::Day("Invalid day too large", 0, 1));
            }
            (_, _, Some(0), Some(0), _, _) => {
                return Err(ParseError::Day("Invalid month too small", 0, 1));
            }
            // (_, _, Some(m1), Some(m2), _, _, ) if m1 * 10 + m2 > 12 => {
            //     return Err(ParseError::Day("Invalid month too large", 2, 3));
            // }
            (_, _, _, _, Some(y1), Some(sep)) if from_separator(&sep)? == 1800 && y1 < 5 => {
                return Err(ParseError::Day("Invalid year before 1850", 4, 7));
            }
            _ => {}
        }

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

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::Syntax(desc, _, _) => write!(f, "Invalid syntax: {}", desc),
            ParseError::Day(desc, _, _) => write!(f, "Invalid day: {}", desc),
            ParseError::Month(desc, _, _) => write!(f, "Invalid month: {}", desc),
            ParseError::Year(desc, _, _) => write!(f, "Invalid year: {}", desc),
            ParseError::Identifier(desc, _, _) => write!(f, "Invalid identifier: {}", desc),
            ParseError::Checksum(_, _, _, ref checksum) => {
                write!(f, "Invalid checksum: expected {}", checksum)
            }
        }
    }
}

impl error::Error for ParseError<'_> {
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

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unable to generate matching hetu")
    }
}

impl error::Error for GenerateError {
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

impl ErrorIndexRange for ParseError<'_> {
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
        _ => panic!("Invalid month {} in year {}", month, year),
    }
}

static CHECKSUM_TABLE: [char; 31] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'H', 'J', 'K',
    'L', 'M', 'N', 'P', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y',
];

fn checksum(ssn: &str) -> char {
    let mut hello: String = ssn[..6].to_string();
    hello.push_str(&ssn[7..10]);
    let nums: usize = hello.parse().unwrap();
    CHECKSUM_TABLE[nums % 31]
}
// fn checksum_num(day: usize, month: usize, year: usize, identifier: usize) -> char {
//     let nums = day * 10_000_000 + month * 100_000 + (year % 100) * 1_000 + identifier;
//     CHECKSUM_TABLE[nums % 31]
// }

fn is_leap_year(year: usize) -> bool {
    ((year % 4 == 0) && (year % 100 != 0)) || (year % 400 == 0)
}

#[cfg(test)]
extern crate regex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_string() {
        assert!(
            Ssn::parse("").unwrap_err() == ParseError::Syntax("Invalid length", 0, 0),
            "fail when given empty String"
        );
    }
    #[test]
    fn test_parse_month_too_large() {
        assert!(
            Ssn::parse("301398-1233").unwrap_err()
                == ParseError::Month("Invalid month number", 2, 4),
            "fail when given birthdate with month out of bounds"
        );
    }
    #[test]
    fn test_parse_day_too_large() {
        assert!(
            Ssn::parse("320198-123P").unwrap_err() == ParseError::Day("Invalid day number", 0, 2),
            "fail when given birthdate with date out of bounds in January"
        );
    }
    #[test]
    fn test_parse_day_too_large_on_non_leap_year() {
        assert!(
            Ssn::parse("290299-123U").unwrap_err() == ParseError::Day("Invalid day number", 0, 2),
            "fail when given birthdate with date out of bounds in February, non leap year"
        );
    }
    #[test]
    fn test_parse_day_too_large_on_leap_year() {
        assert!(
            Ssn::parse("300204-123Y").unwrap_err() == ParseError::Day("Invalid day number", 0, 2),
            "fail when given birth date with date out of bounds in February, a leap year"
        );
    }
    #[test]
    fn test_parse_invalid_year_characters() {
        assert!(
            Ssn::parse("0101AA-123A").unwrap_err() == ParseError::Syntax("Date not integer", 0, 6),
            "fail when given birth date with alphabets"
        );
    }
    #[test]
    fn test_parse_invalid_separator() {
        assert!(
            Ssn::parse("010195_433X").unwrap_err() == ParseError::Syntax("Invalid separator", 6, 7),
            "fail when given invalid separator chars"
        );
    }
    #[test]
    fn test_parse_date_too_long() {
        assert!(
            Ssn::parse("01011995+433X").unwrap_err() == ParseError::Syntax("Invalid length", 0, 13),
            "fail when given too long date"
        );
    }
    #[test]
    fn test_parse_date_too_short() {
        assert!(
            Ssn::parse("01015+433X").unwrap_err() == ParseError::Syntax("Invalid length", 0, 10),
            "fail when given too short date"
        );
    }
    #[test]
    fn test_parse_identifier_too_long() {
        assert!(
            Ssn::parse("010195+4433X").unwrap_err() == ParseError::Syntax("Invalid length", 0, 12),
            "fail when given too long checksum part"
        );
    }
    #[test]
    fn test_parse_identifier_too_short() {
        assert!(
            Ssn::parse("010195+33X").unwrap_err() == ParseError::Syntax("Invalid length", 0, 10),
            "fail when given too long checksum part"
        );
    }
    #[test]
    fn test_parse_male() {
        assert_eq!(
            Ssn::parse("010195+433X").unwrap(),
            Ssn {
                day: 1,
                month: 1,
                year: 1895,
                gender: Gender::Male,
            }
        );
    }
    #[test]
    fn test_parse_female() {
        assert_eq!(
            Ssn::parse("010197-100P").unwrap(),
            Ssn {
                day: 1,
                month: 1,
                year: 1997,
                gender: Gender::Female,
            }
        );
    }
    #[test]
    fn test_parse_1900s() {
        assert_eq!(
            Ssn::parse("010114A173M").unwrap(),
            Ssn {
                day: 1,
                month: 1,
                year: 2014,
                gender: Gender::Male,
            }
        );
    }
    #[test]
    fn test_parse_leap_year() {
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
    }
    #[test]
    fn test_parse_invalid_day_on_leap_year() {
        assert!(
            Ssn::parse("290200-101P").unwrap_err() == ParseError::Day("Invalid day number", 0, 2),
            "fail when given valid finnishSSN with leap year, divisible by 100 and not by 400"
        );
    }
    #[test]
    fn test_parse_leap_year_long() {
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
    }
    #[test]
    fn test_parse_leading_whitespace() {
        assert!(
            Ssn::parse("010114A173M ").unwrap_err() == ParseError::Syntax("Invalid length", 0, 12),
            "fail when given SSN longer than 11 chars, bogus in the end"
        );
    }
    #[test]
    fn test_parse_trailing_whitespace() {
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
    fn test_iter() {
        let pattern = SsnPattern::parse("010197-100P").unwrap();
        let mut iter = Ssn::iter(&pattern);
        let generated = iter.next().unwrap();
        assert!(Ssn::parse(&generated).is_ok());
    }

    #[test]
    fn test_iter_wildcard() {
        let pattern = SsnPattern::parse("???????????").unwrap();
        let mut iter = Ssn::iter(&pattern);
        let generated = iter.next().unwrap();
        assert!(Ssn::parse(&generated).is_ok());
    }

    #[test]
    fn test_iter_fixed_repeated() {
        let pattern = SsnPattern::parse("010197-100P").unwrap();
        let mut iter = Ssn::iter(&pattern);
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn test_iter_wildcard_repeated() {
        let pattern = SsnPattern::parse("?10197-100?").unwrap();
        let mut iter = Ssn::iter(&pattern);
        let first = vec![
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ];
        let second = vec![
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ];
        assert_eq!(first, second);
    }

    #[test]
    fn test_pattern_parse() {
        assert!(SsnPattern::parse("123456-7890").is_ok(), "parse valid SSN");
    }

    #[test]
    fn test_pattern_parse_all_wildcard() {
        assert!(
            SsnPattern::parse("??????-????").is_ok(),
            "parse all wildcard input"
        );
    }

    macro_rules! pattern_parse_failure {
        ($($name:ident: $value:expr,)*) => {$(
            #[test]
            fn $name() {
                let pattern = &SsnPattern::parse($value);
                assert!(pattern.is_err());
            }
        )*}
    }

    pattern_parse_failure! {
        pattern_parse_invalid_checksum: "??????-???O",
        pattern_parse_invalid_year: "????4?+????",
        pattern_parse_month_too_small: "??00??????A",
        // pattern_parse_month_too_large: "??13??????A",
        pattern_parse_day_too_small: "00????????A",
        pattern_parse_day_too_large: "32????????A",
    }

    #[test]
    fn test_generate_never_temporary_identifier_with_wilcard() {
        for _i in 0..1_000_000 {
            let generated = Ssn::generate();
            let first_identifier = generated.chars().nth(7).unwrap();
            assert_ne!(
                first_identifier, '9',
                "never generate identifier in range of 900-999"
            );
        }
    }

    use regex::Regex;

    macro_rules! ssn_generate_success {
        ($($name:ident: $value:expr,)*) => {$(
            #[test]
            fn $name() {
                let pattern = &SsnPattern::parse($value).unwrap();
                let generated = Ssn::generate_by_pattern(pattern).unwrap();
                let matcher = Regex::new($value.replace("?", ".").as_str()).unwrap();
                assert!(matcher.is_match(&generated), "retain expected values");
                assert!(Ssn::parse(&generated).is_ok(), "generate valid SSN");
            }
        )*}
    }

    ssn_generate_success! {
        first_day_of_year_wildcard: "010100-????",
        first_day_of_year_fixed: "010100-???A",
        identifier_smallest_wildcard: "???????002?",
        identifier_smallest_fixed: "???????002A",
        identifier_biggest_wildcard: "???????899?",
        identifier_biggest_fixed: "???????899A",
        decade_smallest_wildcard: "??????+????",
        decade_smallest_fixed: "??????+???A",
        decade_biggest_wildcard: "??????A????",
        decade_biggest_fixed: "??????A???A",
        month_smallest_wildcard: "??01???????",
        month_smallest_fixed: "??01??????A",
        month_biggest_wildcard: "??12???????",
        month_biggest_fixed: "??12??????A",
        day_smallest_wildcard: "01?????????",
        day_smallest_fixed: "01????????A",
        day_biggest_wildcard: "31?????????",
        day_biggest_fixed: "31????????A",
    }

    macro_rules! ssn_generate_failure {
        ($($name:ident: $value:expr,)*) => {$(
            #[test]
            fn $name() {
                let pattern = &SsnPattern::parse($value).unwrap();
                assert!(Ssn::generate_by_pattern(pattern).is_err());
            }
        )*}
    }

    ssn_generate_failure! {
        identifier_too_small_wildcard: "???????001?",
        identifier_too_small_fixed: "???????001A",
        // month_too_small_wildcard: "??00???????",
        // FIXME
        // month_too_small_fixed: "??00??????A",
        // month_too_large_wildcard: "??13???????",
        // FIXME
        // month_too_large_fixed: "??13??????A",
        // day_too_small_wildcard: "00?????????",
        // FIXME
        // day_too_small_fixed: "00????????A",
        // day_too_large_wildcard: "32?????????",
        // day_too_large_fixed: "32????????A",
        day_too_large_on_non_leap_year_wilcard: "290299-????",
        day_too_large_on_non_leap_year_fixed: "290299-???A",
        day_too_large_on_leap_year_wildcard: "300204-????",
        day_too_large_on_leap_year_fixed: "300204-???A",
    }
}
