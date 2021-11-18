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

### Parsing a single text as FIDL
Parsing a text string as single FIDL module works with 
```fipa::parse::parse_module(text: &str)```. The method returns a result of
either an ```fipa::ast::Module``` or a ```nom::Error```.
```rust 
// include the file 'interface.fidl' as text string and parse it
let fidl_text = include_str!("interface.fidl");
match fipa::parser::parse_module(&fidl_text) {
    Ok(module) => { // module contains AST of parsed text },
    Err(error) => { // nom error occured while parsing. },
}
```

### Parsing a set of FIDL files into abstract syntax tree
Multiple FIDL files can be parsed in one rush including recursive parsing
of imported FIDL files. To resolve the imports a list of search directories
can be supplied.

The method used for this is ```fipa::parser::parse_fidls```. It is an 
asynchronous method and parsing of the FIDL files may be done concurrently
to speed up the process. Hence usage of this function requires an 
asynchronous runtime.

```rust
use tokio;
use std::path::Path;

#[tokio::main]
pub async fn main() {
    let fidls = vec![ Path::new("fidl1.fidl"), Path::new("fild2.fidl")];
    let search_dir = vec![Path::new("search_dir")];
    let max_import_nesting = 256usize;

    let (modules, errors) = fipa::compiler::parse_fidls(&fidls, 
                                                        &search_dir, 
                                                        max_import_nesting, 
                                                        true).await;
}
```

The returned tuple is a ```(Vec<(ast::Module, PathBuf)>, Vec<ParseError>)``` 
where the first part contains the successfully parsed files and the second
vector contains the errors that occurred during parsing.

## Capabilities and Limitations
### FIDL Syntax

| Feature                   | Supported | Limitations           |
| :------------------------ | :-------: | :-------------------- |
| TypeCollection            | yes       |                       |
| Constants                 | no        |                       |
| Integers UInt<N>, Int<N>  | yes       |                       | 
| Integer Interval          | no        |                       |
| String                    | yes       |                       |
| Array, Union, Struct      | yes       |                       |
| ByteArray                 | yes       |                       |
| Enumeration               | yes       | enumerator value only as decimal/hex/binary integer |
| Interfaces                | yes       | types supported as in typeCollection         |
| Attributes                | yes       |                       |
| Methods                   | yes       |                       |
| Broadcasts                | yes       |                       |
| Contracts                 | no        |                       |
