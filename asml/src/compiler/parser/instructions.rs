use super::{Parser, ParserError};
use crate::compiler::token::{Token, TokenType};
use asml_vm::opcodes::OpCode;

impl<L: Iterator<Item = Token>> Parser<L> {
    pub(crate) fn parse_no_args(&mut self, c: OpCode) -> Result<(), ParserError> {
        self.prog.append_code(&[c as u8]);
        self.expect_token(TokenType::END_INST)
    }

    pub(crate) fn parse_num(&mut self, c: OpCode) -> Result<(), ParserError> {
        // Arg 1
        self.read_token();
        let val = self.parse_address(1)?;

        self.prog
            .append_code(&[c as u8, (val >> 8) as u8, val as u8]);
        Ok(())
    }

    pub(crate) fn parse_reg(&mut self, c: OpCode) -> Result<(), ParserError> {
        // Arg 1
        self.read_token();
        let reg1 = self.parse_register()?;

        self.prog.append_code(&[c as u8, reg1]);
        Ok(())
    }

    pub(crate) fn parse_reg_reg(&mut self, c: OpCode) -> Result<(), ParserError> {
        // Arg 1
        self.read_token();
        let reg1 = self.parse_register()?;

        // Arg 2
        self.read_token();
        let reg2 = self.parse_register()?;

        self.prog.append_code(&[c as u8, reg1, reg2]);
        Ok(())
    }

    pub(crate) fn parse_reg_num(&mut self, c: OpCode) -> Result<(), ParserError> {
        // Arg 1
        self.read_token();
        let reg1 = self.parse_register()?;

        // Arg 2
        self.read_token();
        let val = self.parse_address(2)?;

        self.prog
            .append_code(&[c as u8, reg1, (val >> 8) as u8, val as u8]);
        Ok(())
    }

    pub(crate) fn parse_reg_half_num(&mut self, c: OpCode) -> Result<(), ParserError> {
        // Arg 1
        self.read_token();
        let reg1 = self.parse_register()?;

        self.read_token();
        self.expect_token(TokenType::IMMEDIATE)?;

        // Arg 2
        self.read_token();
        let val = self.parse_address(2)?;

        if val > 255 {
            Err(self.parse_err("number must be between 0 - 255"))
        } else {
            self.prog.append_code(&[c as u8, reg1, val as u8]);
            Ok(())
        }
    }

    pub(crate) fn parse_inst(
        &mut self,
        imm: OpCode,
        addr: OpCode,
        reg: OpCode,
    ) -> Result<(), ParserError> {
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

    pub(crate) fn parse_inst_no_dest(
        &mut self,
        imm: OpCode,
        addr: OpCode,
        reg: OpCode,
    ) -> Result<(), ParserError> {
        self.read_token();

        match self.cur_tok.name {
            TokenType::NUMBER | TokenType::IDENT => {
                let val = self.parse_address(1)?;
                self.prog
                    .append_code(&[addr as u8, (val >> 8) as u8, val as u8]);
            }
            TokenType::IMMEDIATE => {
                self.read_token();
                let val = self.parse_address(1)?;
                self.prog
                    .append_code(&[imm as u8, (val >> 8) as u8, val as u8]);
            }
            TokenType::REGISTER => {
                let src = self.parse_register()?;
                self.prog.append_code(&[reg as u8, src]);
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

    pub(crate) fn parse_inst_no_imm_no_dest(
        &mut self,
        addr: OpCode,
        reg: OpCode,
    ) -> Result<(), ParserError> {
        self.read_token();

        match self.cur_tok.name {
            TokenType::NUMBER | TokenType::IDENT => {
                let val = self.parse_address(1)?;
                self.prog
                    .append_code(&[addr as u8, (val >> 8) as u8, val as u8]);
            }
            TokenType::REGISTER => {
                let src = self.parse_register()?;
                self.prog.append_code(&[reg as u8, src]);
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

    pub(crate) fn parse_inst_no_imm(
        &mut self,
        addr: OpCode,
        reg: OpCode,
    ) -> Result<(), ParserError> {
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
}
