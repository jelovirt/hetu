# HETU validator and generator in Rust

Simple crate for validating and generating [Finnish Personal Identity Code (HETU)][1].

Works with rust 1.8 stable.

## Usage

Add this to your `Cargo.toml`

```toml
[dependencies.hetu]
git = "https://github.com/jelovirt/hetu.git"
```

To validate:

```rust
extern crate hetu;
use hetu::Ssn::parse;

pub fn main() {
  if parse("121212-121D").is_ok() {
    println!("Valid HETU")
  } else {
    println!("Invalid")
  }
}
```

To generate:

```rust
extern crate hetu;
use hetu::Ssn::generate;

pub fn main() {
  println!("{}", generate())
}
```

## CLI

Command line tool `hetu` can be used to either validate or randomly generate
identifiers.

To validate:

```bash
$ hetu 121212-121D
$ hetu 121212-121C
Error: Invalid checksum
  
  121212-121C
            ^
```

To generate:

```bash
$ hetu
121212-121D
```

[1]: https://en.wikipedia.org/wiki/National_identification_number#Finland