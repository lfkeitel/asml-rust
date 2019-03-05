use std::fmt;
use std::str;

#[allow(non_camel_case_types)]
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
			_ => TokenType::IDENT,
		}
	}

	fn to_string(&self) -> String {
		match self {
			TokenType::ILLEGAL => "ILLEGAL".to_owned(),
			TokenType::EOF => "EOF".to_owned(),
			TokenType::COMMENT => "COMMENT".to_owned(),
			TokenType::END_INST => "END_INST".to_owned(),
			TokenType::COMMA => "COMMA".to_owned(),
			TokenType::IMMEDIATE => "IMMEDIATE".to_owned(),
			TokenType::IDENT => "IDENT".to_owned(),
			TokenType::LABEL => "LABEL".to_owned(),
			TokenType::NUMBER => "NUMBER".to_owned(),
			TokenType::STRING => "STRING".to_owned(),
			TokenType::REGISTER => "REGISTER".to_owned(),
			TokenType::NOOP => "NOOP".to_owned(),
			TokenType::LOAD => "LOAD".to_owned(),
			TokenType::STR => "STR".to_owned(),
			TokenType::XFER => "XFER".to_owned(),
			TokenType::ADD => "ADD".to_owned(),
			TokenType::OR => "OR".to_owned(),
			TokenType::AND => "AND".to_owned(),
			TokenType::XOR => "XOR".to_owned(),
			TokenType::ROTR => "ROTR".to_owned(),
			TokenType::ROTL => "ROTL".to_owned(),
			TokenType::JMP => "JMP".to_owned(),
			TokenType::HALT => "HALT".to_owned(),
			TokenType::JMPA => "JMPA".to_owned(),
			TokenType::LDSP => "LDSP".to_owned(),
			TokenType::PUSH => "PUSH".to_owned(),
			TokenType::POP => "POP".to_owned(),
			TokenType::CALL => "CALL".to_owned(),
			TokenType::RTN => "RTN".to_owned(),
			TokenType::RMB => "RMB".to_owned(),
			TokenType::ORG => "ORG".to_owned(),
			TokenType::FCB => "FCB".to_owned(),
			TokenType::FDB => "FDB".to_owned(),
		}
	}
}

impl fmt::Display for TokenType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.to_string())
	}
}

impl fmt::Debug for TokenType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.to_string())
	}
}

#[derive(Clone)]
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
			line: line,
			col: col,
		}
	}

	pub fn simple(t: TokenType, line: u32, col: u32) -> Self {
		Self::with_literal(t, "".to_string(), line, col)
	}
}
