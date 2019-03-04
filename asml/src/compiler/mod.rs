pub mod lexer;
mod parser;
pub mod token;

use std::fmt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use asml_vm::{Code, CodeSection};
use parser::Parser;

pub enum ParseError {
    Failure(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::Failure(s) => write!(f, "{}", s),
        }
    }
}

pub fn compile_file(filepath: &Path) -> Result<Code, ParseError> {
    let file = File::open(filepath).unwrap();
    let buf = BufReader::new(file);
    let reader = buf.bytes();
    let lex = lexer::Lexer::new(reader);
    let _parser = Parser::new(lex);

    let code: Vec<CodeSection> = Vec::new();

    Ok(code)
}
