use std::collections::HashMap;
use std::fmt;

use super::lexer;
use super::token::{Token, TokenType};

use asml_vm::opcodes::OpCode;

pub enum ParserError {
    InvalidCode(String),
    ExpectedToken(String),
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::InvalidCode(s) => write!(f, "{}", s),
            ParserError::ExpectedToken(s) => write!(f, "{}", s),
        }
    }
}

pub struct Parser {
    lex: lexer::Lexer,
    cur_tok: Token,
    peek_tok: Token,
    prog: Program,
}

impl Parser {
    pub fn new(mut lex: lexer::Lexer) -> Self {
        let cur = lex.next().unwrap();
        let peek = lex.next().unwrap();

        Parser {
            lex: lex,
            cur_tok: cur,
            peek_tok: peek,
            prog: Program::new(),
        }
    }

    pub fn parse(mut self) -> Result<Program, ParserError> {
        while self.cur_tok.name != TokenType::EOF {
            let res = match self.cur_tok.name {
                TokenType::END_INST => break,
                TokenType::LABEL => self.make_label(),

                TokenType::LOAD => self.parse_inst(OpCode::LOADI, OpCode::LOADA, OpCode::LOADR),

                TokenType::STR => self.parse_inst_no_imm(OpCode::STRA, OpCode::STRR),

                // TokenType::XFER => self.ins_movr(),

                // TokenType::ADD => self.ins_add(),
                TokenType::OR => self.parse_inst(OpCode::ORI, OpCode::ORA, OpCode::ORR),
                TokenType::AND => self.parse_inst(OpCode::ANDI, OpCode::ANDA, OpCode::ANDR),
                TokenType::XOR => self.parse_inst(OpCode::XORI, OpCode::XORA, OpCode::XORR),

                // TokenType::ROTR => self.ins_rotr(),
                // TokenType::ROTL => self.ins_rotl(),

                // TokenType::PUSH => self.ins_push(),
                // TokenType::POP => self.ins_pop(),

                // TokenType::CALL => self.ins_call(),

                // TokenType::JMP => self.ins_jmp(),
                // TokenType::JMPA => self.ins_jmpa(),

                // TokenType::LDSP => self.ins_ldsp(),

                // TokenType::HALT => self.ins_halt(),
                // TokenType::NOOP => self.ins_noop(),
                // TokenType::RTN => self.ins_rtn(),

                // TokenType::RMB => self.ins_rmb(),
                // TokenType::ORG => self.ins_org(),
                TokenType::FCB => self.raw_data_fcb(),
                TokenType::FDB => self.raw_data_fdb(),
                _ => Err(ParserError::InvalidCode(format!(
                    "line {}, col {} Unknown token {}",
                    self.cur_tok.line, self.cur_tok.col, self.cur_tok.name
                ))),
            };

            if let Err(e) = res {
                return Err(e);
            }

            self.read_token()
        }

        Ok(self.prog)
    }

    fn read_token(&mut self) {
        self.cur_tok = self.peek_tok;
        self.peek_tok = self.lex.next().unwrap();

        while self.peek_tok.name == TokenType::COMMENT {
            self.peek_tok = self.lex.next().unwrap();
        }
    }

    fn cur_token_is(&self, t: TokenType) -> bool {
        self.cur_tok.name == t
    }

    fn parse_err(&self, msg: &str) -> String {
        format!("{} on line {}", msg, self.cur_tok.line)
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

    fn parse_address(&mut self, pcoffset: u16) -> Result<u16, ParserError> {
        match self.cur_tok.name {
            TokenType::NUMBER => match parse_u16(&self.cur_tok.literal) {
                Some(n) => Ok(n),
                None => Err(ParserError::InvalidCode(self.parse_err("invalid address"))),
            },
            _ => unimplemented!("parse_address"),
        }
    }

    fn make_label(&mut self) -> Result<(), ParserError> {
        self.prog.add_label(&self.cur_tok.literal);
        Ok(())
    }

    fn raw_data_fcb(&mut self) -> Result<(), ParserError> {
        self.read_token();

        while true {
            if !self.cur_token_is(TokenType::NUMBER) && !self.cur_token_is(TokenType::STRING) {
                return Err(ParserError::InvalidCode(
                    self.parse_err("Constant byte must be a number or string"),
                ));
            }

            if self.cur_token_is(TokenType::STRING) {
                self.prog.append_code(self.cur_tok.literal.as_bytes());
            } else if let Some(val) = parse_u16(&self.cur_tok.literal) {
                if val <= 255 {
                    self.prog.append_code(&[val as u8]);
                } else {
                    return Err(ParserError::InvalidCode(self.parse_err("Invalid byte")));
                }
            } else {
                return Err(ParserError::InvalidCode(self.parse_err("Invalid byte")));
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

        while true {
            if self.cur_token_is(TokenType::STRING) {
                return Err(ParserError::InvalidCode(
                    self.parse_err("FDB cannot use a string"),
                ));
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

    fn parse_inst(&mut self, imm: OpCode, addr: OpCode, reg: OpCode) -> Result<(), ParserError> {
        self.read_token();
        let dest = self.parse_register()?;

        self.read_token();
        match self.cur_tok.name {
            TokenType::NUMBER | TokenType::IDENT => {
                let val = self.parse_address(2)?;
                self.prog
                    .append_code(&[addr as u8, dest as u8, (val >> 8) as u8, val as u8]);
            }
            TokenType::IMMEDIATE => {
                self.read_token();
                let val = self.parse_address(2)?;
                self.prog
                    .append_code(&[imm as u8, dest as u8, (val >> 8) as u8, val as u8]);
            }
            TokenType::REGISTER => {
                let src = self.parse_register()?;
                self.prog.append_code(&[reg as u8, dest as u8, src]);
            }
            _ => {
                return Err(self.tokens_err(&[
                    TokenType::NUMBER,
                    TokenType::IDENT,
                    TokenType::IMMEDIATE,
                    TokenType::REGISTER,
                ]));
            }
        };

        self.expect_token(TokenType::END_INST)
    }

    fn parse_inst_no_imm(&mut self, addr: OpCode, reg: OpCode) -> Result<(), ParserError> {
        self.read_token();
        let dest = self.parse_register()?;

        self.read_token();
        match self.cur_tok.name {
            TokenType::NUMBER | TokenType::IDENT => {
                let val = self.parse_address(2)?;
                self.prog
                    .append_code(&[addr as u8, dest as u8, (val >> 8) as u8, val as u8]);
            }
            TokenType::REGISTER => {
                let src = self.parse_register()?;
                self.prog.append_code(&[reg as u8, dest as u8, src]);
            }
            _ => {
                return Err(self.tokens_err(&[
                    TokenType::NUMBER,
                    TokenType::IDENT,
                    TokenType::REGISTER,
                ]));
            }
        };

        self.expect_token(TokenType::END_INST)
    }

    fn parse_register(&self) -> Result<u8, ParserError> {
        if !self.cur_token_is(TokenType::REGISTER) {
            return Err(self.token_err(TokenType::REGISTER));
        }

        let regt = &self.cur_tok;

        match parse_register_lit(&self.cur_tok.literal) {
            Ok(n) => {
                if n > 13 {
                    Err(ParserError::InvalidCode(self.parse_err("invalid register")))
                } else {
                    Ok(n)
                }
            }
            Err(n) => Err(n),
        }
    }
}

fn parse_u16(s: &String) -> Option<u16> {
    if s.starts_with("!") {
        match u16::from_str_radix(s.trim_start_matches("!"), 16) {
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

fn parse_register_lit(s: &String) -> Result<u8, ParserError> {
    match u8::from_str_radix(s.trim_start_matches("0x"), 16) {
        Ok(n) => Ok(n),
        Err(_) => Err(ParserError::InvalidCode("invalid register".to_owned())),
    }
}

pub struct LabelReplace {
    pub label: String,
    pub offset: u16,
}

pub type LabelLinkMap = HashMap<u16, LabelReplace>;
pub type LabelMap = HashMap<String, u16>;

pub struct CodePart {
    pub bytes: Vec<u8>,
    pub start_pc: u16,
    pub pc: u16,
    pub link_map: LabelLinkMap,
}

impl CodePart {
    fn new(pc: u16) -> Self {
        CodePart {
            bytes: Vec::with_capacity(100),
            link_map: HashMap::new(),
            start_pc: pc,
            pc: pc,
        }
    }
}

pub struct Program {
    pub parts: Vec<CodePart>,
    part_i: usize,
    pub labels: LabelMap,
}

impl Program {
    fn new() -> Self {
        Program {
            parts: vec![CodePart::new(0)],
            part_i: 0,
            labels: HashMap::new(),
        }
    }

    fn inc_pc(&mut self) {
        self.parts[self.part_i].pc += 1;
    }

    fn pc(&self) -> u16 {
        self.parts[self.part_i].pc
    }

    fn append_code(&mut self, b: &[u8]) {
        self.parts[self.part_i].bytes.extend_from_slice(b);
    }

    fn add_label(&mut self, name: &String) {
        self.labels
            .insert(name.to_owned(), self.parts[self.part_i].pc);
    }

    fn add_link(&mut self, pc_offset: u16, name: &String, offset: u16) {
        let pc = self.pc() - self.parts[self.part_i].start_pc;
        self.parts[self.part_i].link_map.insert(
            pc,
            LabelReplace {
                label: name.to_owned(),
                offset: offset,
            },
        );
    }

    fn add_code_part(&mut self, pc: u16) {
        self.parts.push(CodePart::new(pc));
        self.part_i += 1;
    }

    fn validate(&mut self) -> Result<(), String> {
        self.parts.sort_by_key(|p| p.start_pc);

        for (i, code) in self.parts.iter().enumerate() {
            if i == self.parts.len() {
                break;
            }

            if code.start_pc + (code.bytes.len() as u16) > self.parts[i + 1].start_pc {
                return Err(format!(
                    "overlapping address regions:
Origin 0x{:04X} goes to 0x{:04X}
Origin 0x{:04X} begins inside region",
                    code.start_pc,
                    code.start_pc + (code.bytes.len() as u16),
                    self.parts[i + 1].start_pc
                ));
            }
        }

        Ok(())
    }
}
