![GitHub Workflow Status](https://img.shields.io/github/workflow/status/alex-seifarth/fipa/Rust)

# FIPA (Franca IDL Parser)

Parser for FRANCA interface definition files (FIDL) and deployment descriptor 
files (FDEPL) in RUST using the **nom** crate.

## Getting Started

### Prerequisites

Assuming a working internet connection to *crates.io* all you need is

* recent RUST environment (rustc, cargo) (see https://www.rust-lang.org/tools/install)

I'm working actually with the most recent RUST version available.

### Building / Running Tests

```
$ git clone https://github.com/alex-seifarth/fipa ./fipa
$ cd fipa
$ cargo build
$ cargo test
```

## Usage

Actually only the library and tests are provided.
