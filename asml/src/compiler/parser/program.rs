use std::collections::HashMap;

use asml_vm::{Code, CodeSection};

#[derive(Debug)]
pub struct LabelReplace {
    pub label: String,
    pub offset: i16,
}

pub type LabelLinkMap = HashMap<u16, LabelReplace>;
pub type LabelMap = HashMap<String, u16>;

#[derive(Debug)]
pub struct CodePart {
    pub bytes: Vec<u8>,
    pub start_pc: u16,
    pc: u16,
    pub link_map: LabelLinkMap,
}

impl CodePart {
    pub fn new(pc: u16) -> Self {
        CodePart {
            bytes: Vec::with_capacity(100),
            link_map: HashMap::new(),
            start_pc: pc,
            pc,
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub parts: Vec<CodePart>,
    part_i: usize,
    pub labels: LabelMap,
}

impl Program {
    pub fn new() -> Self {
        Program {
            parts: vec![CodePart::new(0)],
            part_i: 0,
            labels: HashMap::new(),
        }
    }

    pub fn pc(&self) -> u16 {
        self.parts[self.part_i].pc
    }

    pub fn append_code(&mut self, b: &[u8]) {
        self.parts[self.part_i].bytes.extend_from_slice(b);
        self.parts[self.part_i].pc = self.parts[self.part_i].pc.wrapping_add(b.len() as u16);
    }

    pub fn add_label(&mut self, name: &str) {
        self.labels
            .insert(name.to_owned(), self.parts[self.part_i].pc);
    }

    pub fn add_link(&mut self, pc_offset: u16, name: &str, offset: i16) {
        let pc = self.pc() - self.parts[self.part_i].start_pc;
        self.parts[self.part_i].link_map.insert(
            pc + pc_offset,
            LabelReplace {
                label: name.to_owned(),
                offset,
            },
        );
    }

    pub fn add_code_part(&mut self, pc: u16) {
        self.parts.push(CodePart::new(pc));
        self.part_i += 1;
    }

    pub fn to_code(&self) -> Code {
        let mut code: Vec<CodeSection> = Vec::with_capacity(self.parts.len());

        for part in &self.parts {
            let cs = CodeSection {
                org: part.start_pc,
                code: part.bytes.clone(),
            };

            code.push(cs);
        }

        code
    }

    pub fn validate(&mut self) -> Result<(), String> {
        self.parts.sort_by_key(|p| p.start_pc);

        for (i, code) in self.parts.iter().enumerate() {
            if i == self.parts.len() - 1 {
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

#[cfg(test)]
mod test {
    use super::super::*;
    use crate::compiler::token::{Token, TokenType};

    struct TokenIter {
        tokens: Vec<Token>,
        idx: usize,
    }

    impl TokenIter {
        fn new(tokens: Vec<Token>) -> Self {
            TokenIter { tokens, idx: 0 }
        }
    }

    impl Iterator for TokenIter {
        type Item = Token;

        fn next(&mut self) -> Option<Self::Item> {
            if self.idx == self.tokens.len() {
                Some(Token::simple(TokenType::EOF, 0, 0))
            } else {
                let tok = &self.tokens[self.idx];
                self.idx += 1;
                Some(tok.clone())
            }
        }
    }

    #[test]
    fn address_idents() {
        let tok_stream = TokenIter::new(vec![Token::with_literal(
            TokenType::IDENT,
            "str+2".to_owned(),
            0,
            0,
        )]);
        let mut p = Parser::new(tok_stream);
        let addr = p.parse_address(0).unwrap();
        assert!(addr == 0);

        let replacement = &p.prog.parts[p.prog.part_i].link_map[&0u16];
        assert!(replacement.label == "str");
        assert!(replacement.offset == 2);
    }
}
