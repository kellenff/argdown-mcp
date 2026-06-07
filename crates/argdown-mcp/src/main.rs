//! Argdown MCP server (placeholder binary).
//!
//! For now this just exercises the parser to prove the workspace wires up.
//! The MCP protocol layer is future work.

mod tools;

use argdown_core::Document;
use argdown_parser::parse;

fn main() {
    let source = "";
    match parse(source) {
        Ok(document) => report(&document),
        Err(error) => eprintln!("failed to parse: {error}"),
    }
}

fn report(document: &Document) {
    println!("parsed argdown document: {document:?}");
}
