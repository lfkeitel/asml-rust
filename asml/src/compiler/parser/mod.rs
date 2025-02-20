mod instructions;
pub mod program;

use std::fmt;

use super::token::{Token, TokenType};

use asml_vm::opcodes::OpCode;

use program::*;

pub enum ParserError {
    InvalidCode(String),
    ExpectedToken(String),
    ValidationError(String),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::InvalidCode(s) => write!(f, "{}", s),
            ParserError::ExpectedToken(s) => write!(f, "{}", s),
            ParserError::ValidationError(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::InvalidCode(s) => write!(f, "{}", s),
            ParserError::ExpectedToken(s) => write!(f, "{}", s),
            ParserError::ValidationError(s) => write!(f, "{}", s),
        }
    }
}

pub struct Parser<L: Iterator<Item = Token>> {
    lexer: L,
    cur_tok: Token,
    peek_tok: Token,
    prog: Program,
}

impl<L: Iterator<Item = Token>> Parser<L> {
    pub fn new(mut lexer: L) -> Self {
        let cur = lexer.next().unwrap();
        let peek = lexer.next().unwrap();

        Parser {
            lexer,
            cur_tok: cur,
            peek_tok: peek,
            prog: Program::new(),
        }
    }

    pub fn parse(mut self) -> Result<Program, ParserError> {
        while self.cur_tok.name != TokenType::EOF {
            let res: Result<(), ParserError> = match self.cur_tok.name {
                // Skip empty lines
                TokenType::END_INST | TokenType::COMMENT => {
                    self.read_token();
                    continue;
                }
                TokenType::EOF => break,

                TokenType::LOAD => self.parse_inst(OpCode::LOADI, OpCode::LOADA, OpCode::LOADR),

                TokenType::STR => self.parse_inst_no_imm(OpCode::STRA, OpCode::STRR),

                TokenType::XFER => self.parse_reg_reg(OpCode::XFER),

                TokenType::ADD => self.parse_inst(OpCode::ADDI, OpCode::ADDA, OpCode::ADDR),

                TokenType::OR => self.parse_inst(OpCode::ORI, OpCode::ORA, OpCode::ORR),
                TokenType::AND => self.parse_inst(OpCode::ANDI, OpCode::ANDA, OpCode::ANDR),
                TokenType::XOR => self.parse_inst(OpCode::XORI, OpCode::XORA, OpCode::XORR),

                TokenType::ROTR => self.parse_reg_half_num(OpCode::ROTR),
                TokenType::ROTL => self.parse_reg_half_num(OpCode::ROTL),

                TokenType::PUSH => self.parse_reg(OpCode::PUSH),
                TokenType::POP => self.parse_reg(OpCode::POP),

                TokenType::CALL => self.parse_inst_no_imm_no_dest(OpCode::CALLA, OpCode::CALLR),

                TokenType::JMP => self.parse_reg_num(OpCode::JMP),
                TokenType::JMPA => self.parse_num(OpCode::JMPA),

                TokenType::LDSP => {
                    self.parse_inst_no_dest(OpCode::LDSPI, OpCode::LDSPA, OpCode::LDSPR)
                }

                TokenType::HALT => self.parse_no_args(OpCode::HALT),
                TokenType::NOOP => self.parse_no_args(OpCode::NOOP),
                TokenType::RTN => self.parse_no_args(OpCode::RTN),

                TokenType::DEBUG => self.parse_no_args(OpCode::DEBUG),

                // Meta instructions
                TokenType::LABEL => self.make_label(),
                TokenType::RMB => self.ins_rmb(),
                TokenType::ORG => self.ins_org(),
                TokenType::FCB => self.raw_data_fcb(),
                TokenType::FDB => self.raw_data_fdb(),
                _ => Err(ParserError::InvalidCode(format!(
                    "line {}, col {} Unknown token {}",
                    self.cur_tok.line, self.cur_tok.col, self.cur_tok.name
                ))),
            };

            res?;

            self.read_token()
        }

        match self.prog.validate() {
            Ok(_) => Ok(self.prog),
            Err(e) => Err(ParserError::ValidationError(e)),
        }
    }

    fn read_token(&mut self) {
        self.cur_tok = self.peek_tok.clone();
        self.peek_tok = self.lexer.next().unwrap();
    }

    // Utility methods
    fn cur_token_is(&self, t: TokenType) -> bool {
        self.cur_tok.name == t
    }

    fn parse_err(&self, msg: &str) -> ParserError {
        ParserError::InvalidCode(format!("{} on line {}", msg, self.cur_tok.line))
    }

    fn token_err(&self, t: TokenType) -> ParserError {
        ParserError::ExpectedToken(format!(
            "expected {} on line {}, got {}",
            t, self.cur_tok.line, self.cur_tok.name
        ))
    }

    fn tokens_err(&self, t: &[TokenType]) -> ParserError {
        ParserError::ExpectedToken(format!(
            "expected {:?} on line {}, got {}",
            t, self.cur_tok.line, self.cur_tok.name
        ))
    }

    fn expect_token(&mut self, t: TokenType) -> Result<(), ParserError> {
        self.read_token();
        if !self.cur_token_is(t) {
            Err(self.token_err(t))
        } else {
            Ok(())
        }
    }

    // Meta instructions
    fn make_label(&mut self) -> Result<(), ParserError> {
        self.prog.add_label(&self.cur_tok.literal);
        Ok(())
    }

    fn raw_data_fcb(&mut self) -> Result<(), ParserError> {
        self.read_token();

        loop {
            if !self.cur_token_is(TokenType::NUMBER) && !self.cur_token_is(TokenType::STRING) {
                return Err(self.parse_err("Constant byte must be a number or string"));
            }

            if self.cur_token_is(TokenType::STRING) {
                self.prog.append_code(self.cur_tok.literal.as_bytes());
            } else if let Some(val) = parse_u16(&self.cur_tok.literal) {
                if val <= 255 {
                    self.prog.append_code(&[val as u8]);
                } else {
                    return Err(self.parse_err("Invalid byte"));
                }
            } else {
                return Err(self.parse_err("Invalid byte"));
            }

            self.read_token();
            if self.cur_token_is(TokenType::END_INST) {
                break;
            }

            if !self.cur_token_is(TokenType::COMMA) {
                return Err(self.token_err(TokenType::COMMA));
            }
            self.read_token();
        }

        Ok(())
    }

    fn raw_data_fdb(&mut self) -> Result<(), ParserError> {
        self.read_token();

        loop {
            if self.cur_token_is(TokenType::STRING) {
                return Err(self.parse_err("FDB cannot use a string"));
            }

            let val = self.parse_address(0)?;
            self.prog.append_code(&[(val >> 8) as u8, val as u8]);

            self.read_token();
            if self.cur_token_is(TokenType::END_INST) {
                break;
            }

            if !self.cur_token_is(TokenType::COMMA) {
                return Err(self.token_err(TokenType::COMMA));
            }
            self.read_token();
        }

        Ok(())
    }

    fn ins_rmb(&mut self) -> Result<(), ParserError> {
        self.read_token();
        if !self.cur_token_is(TokenType::NUMBER) {
            return Err(self.token_err(TokenType::NUMBER));
        }

        if let Some(val) = parse_u16(&self.cur_tok.literal) {
            let buf: Vec<u8> = vec![0; val as usize];
            self.prog.append_code(&buf);
            Ok(())
        } else {
            Err(self.parse_err("invalid number literal"))
        }
    }

    fn ins_org(&mut self) -> Result<(), ParserError> {
        self.read_token();
        if !self.cur_token_is(TokenType::NUMBER) {
            return Err(self.token_err(TokenType::NUMBER));
        }

        if let Some(val) = parse_u16(&self.cur_tok.literal) {
            self.prog.add_code_part(val);
            Ok(())
        } else {
            Err(self.parse_err("invalid number literal"))
        }
    }

    // Argument parser methods
    fn parse_address(&mut self, pcoffset: u16) -> Result<u16, ParserError> {
        match self.cur_tok.name {
            TokenType::NUMBER => match parse_u16(&self.cur_tok.literal) {
                Some(n) => Ok(n),
                None => Err(self.parse_err("invalid address")),
            },
            TokenType::STRING => {
                let bytes = self.cur_tok.literal.as_bytes();

                match bytes.len() {
                    0 => Ok(0),
                    1 => Ok(u16::from(bytes[0])),
                    2 => Ok((u16::from(bytes[0]) << 8) + u16::from(bytes[1])),
                    _ => Err(self.parse_err("string too long")),
                }
            }
            TokenType::IDENT => {
                let mut label: &str = self.cur_tok.literal.as_ref();
                let lit = label;

                let mut offset = 0i16;

                let add_index = label.find('+').unwrap_or_default();
                let sub_index = label.find('-').unwrap_or_default();

                if add_index > 0 || sub_index > 0 {
                    let ind = {
                        if add_index > 0 {
                            add_index
                        } else {
                            sub_index
                        }
                    };

                    label = lit.split_at(ind).0;
                    offset = match (lit.split_at(ind).1).parse::<i16>() {
                        Ok(n) => n,
                        Err(_) => return Err(self.parse_err("invalid address offset")),
                    };
                }

                if label == "$" {
                    Ok(self.prog.pc() + offset as u16)
                } else {
                    self.prog.add_link(pcoffset, label, offset);
                    Ok(0)
                }
            }
            _ => unimplemented!("parse_address"),
        }
    }

    fn parse_register(&self) -> Result<u8, ParserError> {
        if !self.cur_token_is(TokenType::REGISTER) {
            return Err(self.token_err(TokenType::REGISTER));
        }

        match parse_register_lit(&self.cur_tok.literal) {
            Ok(n) => {
                if n > 13 {
                    Err(self.parse_err("invalid register"))
                } else {
                    Ok(n)
                }
            }
            Err(n) => Err(n),
        }
    }
}

fn parse_u16(s: &str) -> Option<u16> {
    if s.starts_with('!') {
        match u16::from_str_radix(s.trim_start_matches('!'), 16) {
            Ok(n) => Some(n),
            Err(_) => None,
        }
    } else if s.starts_with("0x") {
        match u16::from_str_radix(s.trim_start_matches("0x"), 16) {
            Ok(n) => Some(n),
            Err(_) => None,
        }
    } else {
        match s.parse::<u16>() {
            Ok(n) => Some(n),
            Err(_) => None,
        }
    }
}

fn parse_register_lit(s: &str) -> Result<u8, ParserError> {
    match u8::from_str_radix(s.trim_start_matches("0x"), 16) {
        Ok(n) => Ok(n),
        Err(_) => Err(ParserError::InvalidCode("invalid register".to_owned())),
    }
}
