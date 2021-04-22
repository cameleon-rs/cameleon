// TODO:
#![allow(unused)]

use std::str::FromStr;

use super::store::ValueData;

#[derive(Debug, PartialEq)]
pub enum Expr {
    BinOp {
        kind: BinOpKind,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    UnOp {
        kind: UnOpKind,
        lhs: Box<Expr>,
    },

    Immediate(ValueData),

    Ident(String),

    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Pow,
    Shl,
    Shr,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    BitAnd,
    BitOr,
    Xor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOpKind {
    Not,
    Neg,
    Sin,
    Cos,
    Tan,
    Asin,
    ACos,
    Atan,
    Abs,
    Expr,
    Ln,
    Lg,
    Sqrt,
    Trunc,
    Floor,
    Ceil,
    Round,
    Sgn,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    LParen,
    RParen,
    Plus,
    Minus,
    Star,
    DoubleStar,
    Slash,
    Percent,
    And,
    DoubleAnd,
    Or,
    DoubleOr,
    Caret,
    Tilde,
    Eq,
    Ne,
    Colon,
    Question,
    Lt,
    Le,
    Gt,
    Ge,
    Shl,
    Shr,
    Ident(String),
    Float(f64),
    Integer(i64),
}

struct Lexer<'a> {
    src: &'a [u8],
    peek: Option<Token>,
    cur: usize,
    peek_char: Option<(char, usize)>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            peek: None,
            cur: 0,
            peek_char: None,
        }
    }

    fn next(&mut self) -> Option<Token> {
        self.peek();
        self.peek.take()
    }

    fn peek(&mut self) -> Option<&Token> {
        if let Some(ref peek) = self.peek {
            return Some(peek);
        }

        while self.eat_char(|c| c.is_whitespace() || c.is_ascii_control()) {}

        self.peek = Some(match self.next_char()? {
            '(' => Token::LParen,
            ')' => Token::RParen,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => {
                if self.eat_char(|c| c == '*') {
                    Token::DoubleStar
                } else {
                    Token::Star
                }
            }
            '/' => Token::Slash,
            '%' => Token::Percent,
            '&' => {
                if self.eat_char(|c| c == '&') {
                    Token::DoubleAnd
                } else {
                    Token::And
                }
            }
            '|' => {
                if self.eat_char(|c| c == '|') {
                    Token::DoubleOr
                } else {
                    Token::Or
                }
            }
            '^' => Token::Caret,
            '~' => Token::Tilde,
            '=' => Token::Eq,
            ':' => Token::Colon,
            '?' => Token::Question,
            '<' => {
                if self.eat_char(|c| c == '>') {
                    Token::Ne
                } else if self.eat_char(|c| c == '=') {
                    Token::Le
                } else if self.eat_char(|c| c == '<') {
                    Token::Shl
                } else {
                    Token::Lt
                }
            }
            '>' => {
                if self.eat_char(|c| c == '=') {
                    Token::Ge
                } else if self.eat_char(|c| c == '>') {
                    Token::Shr
                } else {
                    Token::Gt
                }
            }
            '.' => {
                let start_pos = self.cur - 1;
                while self.eat_char(|c| c.is_numeric()) {}
                let end_pos = self.cur;
                let f = f64::from_str(self.sub_string(start_pos, end_pos)).unwrap();
                Token::Float(f)
            }

            c if c.is_alphabetic() => {
                let start_pos = self.cur - 1;
                while self.eat_char(|c| c.is_alphanumeric() || c == '.') {}
                let end_pos = self.cur;
                Token::Ident(self.sub_string(start_pos, end_pos).into())
            }

            c if c.is_numeric() => {
                if c == '0' && self.eat_char(|c| c == 'x') {
                    let start_pos = self.cur;
                    while self.eat_char(|c| c.is_ascii_hexdigit()) {}
                    let end_pos = self.cur;
                    let i = i64::from_str_radix(self.sub_string(start_pos, end_pos), 16).unwrap();
                    Token::Integer(i)
                } else {
                    let start_pos = self.cur - 1;
                    let mut is_integer = true;
                    while self.eat_char(|c| {
                        if c == '.' {
                            is_integer = false;
                            true
                        } else {
                            c.is_numeric()
                        }
                    }) {}
                    let end_pos = self.cur;
                    let s = self.sub_string(start_pos, end_pos);
                    if is_integer {
                        Token::Integer(i64::from_str(s).unwrap())
                    } else {
                        Token::Float(f64::from_str(s).unwrap())
                    }
                }
            }

            c => panic!("unexpected character `{}` in formula", c),
        });

        self.peek.as_ref()
    }

    fn next_char(&mut self) -> Option<char> {
        self.peek_char();
        if let Some((peek, idx)) = self.peek_char.take() {
            self.cur = idx;
            Some(peek)
        } else {
            None
        }
    }

    fn eat_char(&mut self, f: impl FnOnce(char) -> bool) -> bool {
        let peek = match self.peek_char() {
            Some(peek) => peek,
            None => return false,
        };
        if f(peek) {
            self.next_char();
            true
        } else {
            false
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        if let Some((peek, _)) = self.peek_char {
            return Some(peek);
        }

        let (peek, idx) = if self.peek_char_raw('&', 0)
            && self.peek_char_raw('a', 1)
            && self.peek_char_raw('m', 2)
            && self.peek_char_raw('p', 3)
            && self.peek_char_raw(';', 4)
        {
            ('&', self.cur + 5)
        } else if self.peek_char_raw('&', 0)
            && self.peek_char_raw('l', 1)
            && self.peek_char_raw('t', 2)
            && self.peek_char_raw(';', 3)
        {
            ('<', self.cur + 4)
        } else if self.peek_char_raw('&', 0)
            && self.peek_char_raw('g', 1)
            && self.peek_char_raw('t', 2)
            && self.peek_char_raw(';', 3)
        {
            ('>', self.cur + 4)
        } else {
            if let Some(c) = self.src.get(self.cur).map(|c| *c as char) {
                (c, self.cur + 1)
            } else {
                return None;
            }
        };

        self.peek_char = Some((peek, idx));
        Some(peek)
    }

    fn peek_char_raw(&self, c: char, n: usize) -> bool {
        if let Some(next) = self.src.get(self.cur + n).map(|c| *c as char) {
            next == c
        } else {
            false
        }
    }

    fn sub_string(&self, start_pos: usize, end_pos: usize) -> &str {
        std::str::from_utf8(&self.src[start_pos..end_pos]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let t = Lexer::new("&amp;").next().unwrap();
        assert_eq!(Token::And, t);

        let t = Lexer::new("&lt;").next().unwrap();
        assert_eq!(Token::Lt, t);

        let t = Lexer::new("&gt;").next().unwrap();
        assert_eq!(Token::Gt, t);

        let t = Lexer::new("Foo1.Max").next().unwrap();
        assert_eq!(Token::Ident("Foo1.Max".into()), t);

        let t = Lexer::new("0xa").next().unwrap();
        assert_eq!(Token::Integer(0xa), t);

        let t = Lexer::new("10").next().unwrap();
        assert_eq!(Token::Integer(10), t);

        let t = Lexer::new("0.1").next().unwrap();
        assert!(matches!(t, Token::Float(_)));

        let t = Lexer::new(".1").next().unwrap();
        assert!(matches!(t, Token::Float(_)));

        let t = Lexer::new("  10 ").next().unwrap();
        assert_eq!(Token::Integer(10), t);

        let mut lexer = Lexer::new("&&||<>**>><<");
        assert_eq!(Token::DoubleAnd, lexer.next().unwrap());
        assert_eq!(Token::DoubleOr, lexer.next().unwrap());
        assert_eq!(Token::Ne, lexer.next().unwrap());
        assert_eq!(Token::DoubleStar, lexer.next().unwrap());
        assert_eq!(Token::Shr, lexer.next().unwrap());
        assert_eq!(Token::Shl, lexer.next().unwrap());
    }
}
