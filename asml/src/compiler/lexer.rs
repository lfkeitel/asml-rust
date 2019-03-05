use std::fmt::Write;
use std::io;

use super::token::Token as token;
use super::token::TokenType as tokent;

pub struct Lexer {
    reader: Box<dyn Iterator<Item = Result<u8, io::Error>>>,
    cur_ch: u8,
    peek_ch: u8,
    line: u32,
    col: u32,
}

impl Lexer {
    pub fn new<I: 'static>(src: I) -> Self
    where
        I: Iterator<Item = Result<u8, io::Error>>,
    {
        let mut l = Lexer {
            reader: Box::new(src),
            cur_ch: 0,
            peek_ch: 0,
            line: 1,
            col: 0,
        };

        l.read_char();
        l.read_char();
        l
    }

    fn read_char(&mut self) {
        self.cur_ch = self.peek_ch;
        self.peek_ch = self.reader.next().unwrap_or(Ok(0)).unwrap_or(0);

        if self.peek_ch == b'\r' {
            self.peek_ch = self.reader.next().unwrap_or(Ok(0)).unwrap_or(0);
        }

        self.col += 1;
    }

    fn reset_pos(&mut self) {
        self.line += 1;
        self.col = 0;
    }

    fn devour_whitespace(&mut self) {
        while is_whitespace(self.cur_ch) {
            self.read_char();
        }
    }

    fn read_identifier(&mut self) -> String {
        let mut ident = String::new();
        while is_ident(self.cur_ch) {
            ident.write_char(char::from(self.cur_ch)).unwrap();
            self.read_char();
        }
        ident
    }

    fn read_string(&mut self) -> String {
        // TODO: should probably support escape sequences

        self.read_char(); // Go over opening quote
        let mut ident = String::new();
        while self.cur_ch != b'"' {
            ident.write_char(char::from(self.cur_ch)).unwrap();
            self.read_char();
        }
        ident
    }

    fn read_single_line_comment(&mut self) -> String {
        self.read_char(); // Go over opening quote
        let mut comm = String::new();
        while self.cur_ch != b'\n' && self.cur_ch != 0 {
            comm.write_char(char::from(self.cur_ch)).unwrap();
            self.read_char();
        }
        comm.trim().to_owned()
    }

    fn read_number(&mut self) -> String {
        let mut num = String::new();
        while is_digit(self.cur_ch) || is_hex_digit(self.cur_ch) || self.cur_ch == b'!' {
            num.write_char(char::from(self.cur_ch)).unwrap();
            self.read_char();
        }
        num
    }
}

impl Iterator for Lexer {
    type Item = token;

    fn next(&mut self) -> Option<Self::Item> {
        macro_rules! some_token {
            ($inst:expr) => {{
                Some(token::simple($inst, self.line, self.col))
            }};

            ($inst:expr, $s:expr) => {{
                Some(token::with_literal($inst, $s, self.line, self.col))
            }};

            ($inst:expr, $s:expr, $col:expr) => {{
                Some(token::with_literal($inst, $s, self.line, $col))
            }};
        }

        if self.cur_ch == b'\n' {
            let t = token::simple(tokent::END_INST, self.line, self.col);
            self.reset_pos();
            self.read_char();
            return Some(t);
        }

        self.devour_whitespace();

        let tok = match self.cur_ch {
            b':' => {
                self.read_char();
                some_token!(tokent::LABEL, self.read_identifier())
            }
            b'#' => some_token!(tokent::IMMEDIATE),
            b',' => some_token!(tokent::COMMA),
            b'"' => some_token!(tokent::STRING, self.read_string()),
            b';' => {
                let col = self.col;
                let t = some_token!(tokent::COMMENT, self.read_single_line_comment(), col);
                self.reset_pos();
                t
            }
            b'%' => {
                self.read_char();
                if self.cur_ch == b'S' && self.peek_ch == b'P' {
                    self.read_char();
                    some_token!(tokent::REGISTER, "SP".to_owned())
                } else {
                    some_token!(
                        tokent::REGISTER,
                        String::from_utf8(vec![self.cur_ch]).unwrap()
                    )
                }
            }
            0 => some_token!(tokent::EOF),
            _ => {
                if is_letter(self.cur_ch) {
                    let lit = self.read_identifier();
                    let tt = tokent::lookup_ident(&lit);
                    return some_token!(tt, lit);
                } else if is_digit(self.cur_ch) {
                    return some_token!(tokent::NUMBER, self.read_number());
                } else {
                    some_token!(tokent::ILLEGAL)
                }
            }
        };

        self.read_char();
        tok
    }
}

fn is_whitespace(ch: u8) -> bool {
    ch == b' ' || ch == b'\t' || ch == b'\n' || ch == b'\r'
}

fn is_ident(ch: u8) -> bool {
    is_letter(ch) || is_digit(ch) || ch == b'-' || ch == b'+'
}

fn is_letter(ch: u8) -> bool {
    b'a' <= ch && ch <= b'z' || b'A' <= ch && ch <= b'Z' || ch == b'_' || ch == b'$'
}

fn is_hex_digit(ch: u8) -> bool {
    b'a' <= ch && ch <= b'f' || b'A' <= ch && ch <= b'F' || ch == b'x'
}

fn is_digit(ch: u8) -> bool {
    b'0' <= ch && ch <= b'9'
}
