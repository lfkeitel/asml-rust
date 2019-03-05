mod lexer;
mod linker;
mod parser;
mod token;

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use asml_vm::Code;
use parser::{Parser, ParserError};

pub fn compile_file(filepath: &Path) -> Result<Code, ParserError> {
    let file = File::open(filepath).unwrap();
    let buf = BufReader::new(file);
    let reader = buf.bytes();
    let lex = lexer::Lexer::new(reader);
    let mut prog = Parser::new(lex).parse()?;

    if let Err(s) = linker::link(&mut prog) {
        Err(ParserError::InvalidCode(s))
    } else {
        Ok(prog.to_code())
    }
}
