use std::fmt;
use std::str;

#[allow(non_camel_case_types,clippy::upper_case_acronyms)]
#[derive(Copy, Clone, PartialEq)]
pub enum TokenType {
    ILLEGAL,
    EOF,
    COMMENT,
    END_INST,
    COMMA,
    IMMEDIATE,
    IDENT,
    LABEL,
    NUMBER,
    STRING,
    REGISTER,
    NOOP,
    LOAD,
    STR,
    XFER,
    ADD,
    OR,
    AND,
    XOR,
    ROTR,
    ROTL,
    JMP,
    HALT,
    JMPA,
    LDSP,
    PUSH,
    POP,
    CALL,
    RTN,
    RMB,
    ORG,
    FCB,
    FDB,
    DEBUG,
}

impl TokenType {
    pub fn lookup_ident(s: &str) -> Self {
        match s {
            "NOOP" => TokenType::NOOP,
            "LOAD" => TokenType::LOAD,
            "STR" => TokenType::STR,
            "XFER" => TokenType::XFER,
            "ADD" => TokenType::ADD,
            "OR" => TokenType::OR,
            "AND" => TokenType::AND,
            "XOR" => TokenType::XOR,
            "ROTR" => TokenType::ROTR,
            "ROTL" => TokenType::ROTL,
            "JMP" => TokenType::JMP,
            "HALT" => TokenType::HALT,
            "JMPA" => TokenType::JMPA,
            "LDSP" => TokenType::LDSP,
            "PUSH" => TokenType::PUSH,
            "POP" => TokenType::POP,
            "CALL" => TokenType::CALL,
            "RTN" => TokenType::RTN,
            "RMB" => TokenType::RMB,
            "ORG" => TokenType::ORG,
            "FCB" => TokenType::FCB,
            "FDB" => TokenType::FDB,
            "DEBUG" => TokenType::DEBUG,
            _ => TokenType::IDENT,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenType::ILLEGAL => "ILLEGAL",
                TokenType::EOF => "EOF",
                TokenType::COMMENT => "COMMENT",
                TokenType::END_INST => "END_INST",
                TokenType::COMMA => "COMMA",
                TokenType::IMMEDIATE => "IMMEDIATE",
                TokenType::IDENT => "IDENT",
                TokenType::LABEL => "LABEL",
                TokenType::NUMBER => "NUMBER",
                TokenType::STRING => "STRING",
                TokenType::REGISTER => "REGISTER",
                TokenType::NOOP => "NOOP",
                TokenType::LOAD => "LOAD",
                TokenType::STR => "STR",
                TokenType::XFER => "XFER",
                TokenType::ADD => "ADD",
                TokenType::OR => "OR",
                TokenType::AND => "AND",
                TokenType::XOR => "XOR",
                TokenType::ROTR => "ROTR",
                TokenType::ROTL => "ROTL",
                TokenType::JMP => "JMP",
                TokenType::HALT => "HALT",
                TokenType::JMPA => "JMPA",
                TokenType::LDSP => "LDSP",
                TokenType::PUSH => "PUSH",
                TokenType::POP => "POP",
                TokenType::CALL => "CALL",
                TokenType::RTN => "RTN",
                TokenType::RMB => "RMB",
                TokenType::ORG => "ORG",
                TokenType::FCB => "FCB",
                TokenType::FDB => "FDB",
                TokenType::DEBUG => "DEBUG",
            }
        )
    }
}

impl fmt::Debug for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub name: TokenType,
    pub literal: String,
    pub line: u32,
    pub col: u32,
}

impl Token {
    pub fn with_literal(t: TokenType, lit: String, line: u32, col: u32) -> Self {
        Token {
            name: t,
            literal: lit,
            line,
            col,
        }
    }

    pub fn simple(t: TokenType, line: u32, col: u32) -> Self {
        Self::with_literal(t, "".to_string(), line, col)
    }
}
