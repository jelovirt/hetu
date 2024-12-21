# Personal identity code validator and generator in Rust

[![Crates.io](https://img.shields.io/crates/v/hetu.svg)](https://crates.io/crates/hetu)
[![Tests](https://github.com/jelovirt/hetu/actions/workflows/test.yml/badge.svg)](https://github.com/jelovirt/hetu/actions/workflows/test.yml)

Simple crate for validating and generating [Finnish Personal Identity Code][1] (henkilötunnus, <abbr>hetu</abbr>).

Supports the [1.1.2023 format](https://dvv.fi/hetu-uudistus).

Works with rust 1.8 stable.

## Usage

Add this to your `Cargo.toml`

```toml
[dependencies.hetu]
git = "https://github.com/jelovirt/hetu.git"
```

To validate a personal identity code:

```rust
extern crate hetu;
use hetu::Ssn;

pub fn main() {
    if Ssn::parse("121212-121D").is_ok() {
        println!("Valid")
    } else {
        println!("Invalid")
    }
}
```

To generate a personal identity code:

```rust
extern crate hetu;
use hetu::Ssn;

pub fn main() {
    println!("{}", Ssn::generate());
}
```

To generate a personal identity code by pattern:

```rust
extern crate hetu;
use hetu::Ssn;
use hetu::SsnPattern;

pub fn main() {
    let pattern = SsnPattern::parse("111111-111?").unwrap();
    println!("{}", Ssn::generate_by_pattern(pattern).unwrap());
}
```

## CLI

Command line tool `hetu` can be used to either validate or randomly generate
personal identity codes.

To validate a personal identity code:

```bash
$ hetu 121212-121D
$ echo 121212-121D | hetu -
$ hetu 121212-121C
Error: Invalid checksum: expected D
  
  121212-121C
            ^
```

To generate a personal identity code:

```bash
$ hetu
121212-121D
```

To generate a personal identity code by pattern that can contain wildcards:

```bash
$ hetu -p "121212-121?"
121212-121D
$ hetu -p "121212-???D"
121212-028D
$ hetu -p "??????-???D"
241151-028D
```

[1]: https://dvv.fi/en/personal-identity-code
