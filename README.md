# HETU validator and generator in Rust

Simple crate for validating and generating Finnish Personal Identity Code (HETU).

Works with rust 1.8 stable.

##  Usage

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
