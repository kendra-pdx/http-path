# URL Path Tools

## Features
- Simple URL path parser
- Path matcher / extractor from strongly typed patterns
  - supports extracting &str and u32 from the source path
- `extractor!` macro for simplifying matcher expressions

## Principles
- `#![no_std]` intended for embeddded applications.
- Simple and lightweight. Not the most robust, but gets the job done well.
- Zero-copy wherever possible.

## Usage
```rust
use http-path::prelude::*;
```

- See [usage.rs](./tests/usage.rs) for a slightly more detailed example.
- See [the grammar](./crates/macros/src/extractor/extractor.pest) for the macro expression syntax.
