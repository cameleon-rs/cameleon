#![allow(
    clippy::missing_panics_doc,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]

use std::{borrow::Borrow, collections::HashMap, fmt, hash::Hash, str::FromStr};

use tracing::debug;

use super::{GenApiError, GenApiResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    pub(crate) expr: Expr,
}

impl Formula {
    #[must_use]
    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    pub fn eval<K, V>(&self, var_env: &HashMap<K, V>) -> GenApiResult<EvaluationResult>
    where
        K: Borrow<str> + Eq + Hash + fmt::Debug,
        V: Borrow<Expr> + fmt::Debug,
    {
        self.expr.eval(var_env)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    BinOp {
        kind: BinOpKind,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    UnOp {
        kind: UnOpKind,
        expr: Box<Expr>,
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    Integer(i64),
    Float(f64),
    Ident(String),
}

impl From<i64> for Expr {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<f64> for Expr {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for Expr {
    fn from(b: bool) -> Self {
        if b {
            Self::Integer(1)
        } else {
            Self::Integer(0)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EvaluationResult {
    Integer(i64),
    Float(f64),
}

impl From<i64> for EvaluationResult {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<f64> for EvaluationResult {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for EvaluationResult {
    fn from(b: bool) -> Self {
        if b {
            Self::Integer(1)
        } else {
            Self::Integer(0)
        }
    }
}

impl EvaluationResult {
    #[must_use]
    pub fn as_integer(self) -> i64 {
        match self {
            Self::Integer(i) => i,
            Self::Float(f) => f as i64,
        }
    }

    #[must_use]
    pub fn as_float(self) -> f64 {
        match self {
            Self::Integer(i) => i as f64,
            Self::Float(f) => f,
        }
    }

    #[must_use]
    pub fn as_bool(self) -> bool {
        match self {
            Self::Integer(i) => i != 0,
            Self::Float(f) => f != 0.,
        }
    }

    fn is_integer(&self) -> bool {
        matches!(self, Self::Integer(..))
    }
}

impl Expr {
    #[must_use]
    #[tracing::instrument(level = "trace")]
    pub fn eval<K, V>(&self, var_env: &HashMap<K, V>) -> GenApiResult<EvaluationResult>
    where
        K: Borrow<str> + Eq + Hash + fmt::Debug,
        V: Borrow<Expr> + fmt::Debug,
    {
        match self {
            Self::BinOp { kind, lhs, rhs } => lhs.eval_binop(*kind, rhs, var_env),
            Self::UnOp { kind, expr } => expr.eval_unop(*kind, var_env),
            Self::If { cond, then, else_ } => {
                if cond.eval(var_env)?.as_bool() {
                    then.eval(var_env)
                } else {
                    else_.eval(var_env)
                }
            }
            &Self::Integer(i) => Ok(i.into()),
            &Self::Float(f) => Ok(f.into()),
            Self::Ident(s) => var_env
                .get(s.as_str())
                .ok_or_else(|| {
                    GenApiError::invalid_node(
                        format!("ident not found in variable env: {} not found", s).into(),
                    )
                })?
                .borrow()
                .eval(var_env),
        }
    }

    fn eval_binop<K, V>(
        &self,
        op: BinOpKind,
        rhs: &Self,
        var_env: &HashMap<K, V>,
    ) -> GenApiResult<EvaluationResult>
    where
        K: Borrow<str> + Eq + Hash + fmt::Debug,
        V: Borrow<Expr> + fmt::Debug,
    {
        use std::ops::{Add, Div, Mul, Rem, Sub};

        Ok(match op {
            BinOpKind::And => {
                (self.eval(var_env)?.as_bool() && rhs.eval(var_env)?.as_bool()).into()
            }
            BinOpKind::Or => (self.eval(var_env)?.as_bool() || rhs.eval(var_env)?.as_bool()).into(),

            _ => {
                let lhs = self.eval(var_env)?;
                let rhs = rhs.eval(var_env)?;

                macro_rules! apply_arithmetic_op {
                    ($fint:ident, $ffloat:ident) => {{
                        if lhs.is_integer() && rhs.is_integer() {
                            (lhs.as_integer().$fint(rhs.as_integer())).0.into()
                        } else {
                            (lhs.as_float().$ffloat(rhs.as_float())).into()
                        }
                    }};
                }

                macro_rules! apply_cmp_op {
                    ($fint:ident, $ffloat:ident) => {{
                        if lhs.is_integer() && rhs.is_integer() {
                            (lhs.as_integer().$fint(&rhs.as_integer())).into()
                        } else {
                            (lhs.as_float().$ffloat(&rhs.as_float())).into()
                        }
                    }};
                }
                match op {
                    BinOpKind::Add => apply_arithmetic_op!(overflowing_add, add),
                    BinOpKind::Sub => apply_arithmetic_op!(overflowing_sub, sub),
                    BinOpKind::Mul => apply_arithmetic_op!(overflowing_mul, mul),
                    BinOpKind::Div => apply_arithmetic_op!(overflowing_div, div),
                    BinOpKind::Rem => apply_arithmetic_op!(overflowing_rem, rem),
                    BinOpKind::Pow => {
                        if lhs.is_integer() && rhs.is_integer() {
                            lhs.as_integer()
                                .overflowing_pow(rhs.as_integer() as u32)
                                .0
                                .into()
                        } else {
                            lhs.as_float().powf(rhs.as_float()).into()
                        }
                    }
                    BinOpKind::Eq => apply_cmp_op!(eq, eq),
                    BinOpKind::Ne => apply_cmp_op!(ne, ne),
                    BinOpKind::Lt => apply_cmp_op!(lt, lt),
                    BinOpKind::Le => apply_cmp_op!(le, le),
                    BinOpKind::Gt => apply_cmp_op!(gt, gt),
                    BinOpKind::Ge => apply_cmp_op!(ge, ge),
                    BinOpKind::Shl => lhs
                        .as_integer()
                        .overflowing_shl(rhs.as_integer() as u32)
                        .0
                        .into(),
                    BinOpKind::Shr => lhs
                        .as_integer()
                        .overflowing_shr(rhs.as_integer() as u32)
                        .0
                        .into(),
                    BinOpKind::BitAnd => (lhs.as_integer() & rhs.as_integer()).into(),
                    BinOpKind::BitOr => (lhs.as_integer() | rhs.as_integer()).into(),
                    BinOpKind::Xor => (lhs.as_integer() ^ rhs.as_integer()).into(),
                    _ => unreachable!(),
                }
            }
        })
    }

    fn eval_unop<K, V>(
        &self,
        op: UnOpKind,
        var_env: &HashMap<K, V>,
    ) -> GenApiResult<EvaluationResult>
    where
        K: Borrow<str> + Eq + Hash + fmt::Debug,
        V: Borrow<Expr> + fmt::Debug,
    {
        use std::ops::Neg;

        let res = self.eval(&var_env)?;
        macro_rules! apply_op {
            ($f:ident) => {
                match res {
                    EvaluationResult::Integer(i) => EvaluationResult::from(i.$f()),
                    EvaluationResult::Float(f) => EvaluationResult::from(f.$f()),
                }
            };
        }

        Ok(match op {
            UnOpKind::Not => (!res.as_integer()).into(),
            UnOpKind::Abs => apply_op!(abs),
            UnOpKind::Sgn => apply_op!(signum),
            UnOpKind::Neg => apply_op!(neg),
            UnOpKind::Sin => res.as_float().sin().into(),
            UnOpKind::Cos => res.as_float().cos().into(),
            UnOpKind::Tan => res.as_float().tan().into(),
            UnOpKind::Asin => res.as_float().asin().into(),
            UnOpKind::Acos => res.as_float().acos().into(),
            UnOpKind::Atan => res.as_float().atan().into(),
            UnOpKind::Exp => res.as_float().exp().into(),
            UnOpKind::Ln => res.as_float().ln().into(),
            UnOpKind::Lg => res.as_float().log10().into(),
            UnOpKind::Sqrt => res.as_float().sqrt().into(),
            UnOpKind::Trunc => res.as_float().trunc().into(),
            UnOpKind::Floor => res.as_float().floor().into(),
            UnOpKind::Ceil => res.as_float().ceil().into(),
            UnOpKind::Round => res.as_float().round().into(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    Shl,
    Shr,
    And,
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
    Abs,
    Sgn,
    Neg,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Exp,
    Ln,
    Lg,
    Sqrt,
    Trunc,
    Floor,
    Ceil,
    Round,
}

#[must_use]
#[tracing::instrument(level = "trace")]
pub fn parse(s: &str) -> Expr {
    debug!("start parsing expression in `formula`");
    let lexer = Lexer::new(s);
    Parser { lexer }.expr()
}

struct Parser<'a> {
    lexer: Lexer<'a>,
}

macro_rules! parse_binop {
    ($self:ident.$f:ident, ($token:expr, $op:expr) $(,($token_rep:expr, $op_rep:expr))*) => {
        {
        let mut expr = $self.$f();
        loop {
            let (op_kind, rhs) = if $self.eat(&$token) {
                ($op, $self.$f())
            } $(else if $self.eat(&$token_rep) {
                ($op_rep, $self.$f())
            })* else {
                break;
            };
            expr = Expr::BinOp {
                kind: op_kind,
                lhs: expr.into(),
                rhs: rhs.into(),
            };
        }
        expr
        }
    }
}

impl<'a> Parser<'a> {
    fn expr(&mut self) -> Expr {
        let expr = self.logical_or();
        if self.eat(&Token::Question) {
            let then = self.expr();
            self.expect(&Token::Colon);
            let else_ = self.expr();
            Expr::If {
                cond: expr.into(),
                then: then.into(),
                else_: else_.into(),
            }
        } else {
            expr
        }
    }

    fn logical_or(&mut self) -> Expr {
        parse_binop!(self.logical_and, (Token::DoubleOr, BinOpKind::Or))
    }

    fn logical_and(&mut self) -> Expr {
        parse_binop!(self.bitwise_or, (Token::DoubleAnd, BinOpKind::And))
    }

    fn bitwise_or(&mut self) -> Expr {
        parse_binop!(self.bitwise_xor, (Token::Or, BinOpKind::BitOr))
    }

    fn bitwise_xor(&mut self) -> Expr {
        parse_binop!(self.bitwise_and, (Token::Caret, BinOpKind::Xor))
    }

    fn bitwise_and(&mut self) -> Expr {
        parse_binop!(self.eq, (Token::And, BinOpKind::BitAnd))
    }

    fn eq(&mut self) -> Expr {
        parse_binop!(
            self.rel,
            (Token::Eq, BinOpKind::Eq),
            (Token::Ne, BinOpKind::Ne)
        )
    }

    fn rel(&mut self) -> Expr {
        parse_binop!(
            self.bit_shift,
            (Token::Lt, BinOpKind::Lt),
            (Token::Le, BinOpKind::Le),
            (Token::Gt, BinOpKind::Gt),
            (Token::Ge, BinOpKind::Ge)
        )
    }

    fn bit_shift(&mut self) -> Expr {
        parse_binop!(
            self.term,
            (Token::Shl, BinOpKind::Shl),
            (Token::Shr, BinOpKind::Shr)
        )
    }

    fn term(&mut self) -> Expr {
        parse_binop!(
            self.factor,
            (Token::Plus, BinOpKind::Add),
            (Token::Minus, BinOpKind::Sub)
        )
    }

    fn factor(&mut self) -> Expr {
        parse_binop!(
            self.not,
            (Token::Star, BinOpKind::Mul),
            (Token::Slash, BinOpKind::Div),
            (Token::Percent, BinOpKind::Rem)
        )
    }

    fn not(&mut self) -> Expr {
        if self.eat(&Token::Tilde) {
            let expr = self.not();
            Expr::UnOp {
                kind: UnOpKind::Not,
                expr: expr.into(),
            }
        } else if self.eat(&Token::Minus) {
            let expr = self.not();
            Expr::UnOp {
                kind: UnOpKind::Neg,
                expr: expr.into(),
            }
        } else {
            // Eat unary `+` if exists.
            self.eat(&Token::Plus);
            self.pow()
        }
    }

    fn pow(&mut self) -> Expr {
        let expr = self.call();
        if self.eat(&Token::DoubleStar) {
            let rhs = self.pow();
            Expr::BinOp {
                kind: BinOpKind::Pow,
                lhs: expr.into(),
                rhs: rhs.into(),
            }
        } else {
            expr
        }
    }

    fn call(&mut self) -> Expr {
        if let Some(op_kind) = self.next_call() {
            self.expect(&Token::LParen);
            let expr = self.expr();
            self.expect(&Token::RParen);
            Expr::UnOp {
                kind: op_kind,
                expr: expr.into(),
            }
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        if self.eat(&Token::LParen) {
            let expr = self.expr();
            self.expect(&Token::RParen);
            expr
        } else if let Some(i) = self.next_integer() {
            Expr::Integer(i)
        } else if let Some(f) = self.next_float() {
            Expr::Float(f)
        } else {
            let s = self.next_ident().unwrap();
            Expr::Ident(s)
        }
    }

    fn eat(&mut self, tok: &Token) -> bool {
        match self.lexer.peek() {
            Some(peek) if peek == tok => {
                self.lexer.next();
                true
            }
            _ => false,
        }
    }

    fn next_call(&mut self) -> Option<UnOpKind> {
        let s = match self.lexer.peek() {
            Some(Token::Ident(s)) => s,
            _ => return None,
        };
        let op = Some(match s.as_str() {
            "NEG" => UnOpKind::Neg,
            "SIN" => UnOpKind::Sin,
            "COS" => UnOpKind::Cos,
            "TAN" => UnOpKind::Tan,
            "ASIN" => UnOpKind::Asin,
            "ACOS" => UnOpKind::Acos,
            "ATAN" => UnOpKind::Atan,
            "ABS" => UnOpKind::Abs,
            "EXP" => UnOpKind::Exp,
            "LN" => UnOpKind::Ln,
            "LG" => UnOpKind::Lg,
            "SQRT" => UnOpKind::Sqrt,
            "TRUNC" => UnOpKind::Trunc,
            "FLOOR" => UnOpKind::Floor,
            "CEIL" => UnOpKind::Ceil,
            "ROUND" => UnOpKind::Round,
            _ => return None,
        });

        self.lexer.next();
        op
    }

    fn next_integer(&mut self) -> Option<i64> {
        if let Some(&Token::Integer(i)) = self.lexer.peek() {
            self.lexer.next();
            Some(i)
        } else {
            None
        }
    }

    fn next_float(&mut self) -> Option<f64> {
        if let Some(&Token::Float(f)) = self.lexer.peek() {
            self.lexer.next();
            Some(f)
        } else if let Some(Token::Ident(s)) = self.lexer.peek() {
            let f = match s.as_str() {
                "PI" => std::f64::consts::PI,
                "E" => std::f64::consts::E,
                _ => return None,
            };
            self.lexer.next();
            Some(f)
        } else {
            None
        }
    }

    fn next_ident(&mut self) -> Option<String> {
        if let Some(Token::Ident(s)) = self.lexer.peek() {
            let s = s.to_string();
            self.lexer.next();
            Some(s)
        } else {
            None
        }
    }

    fn expect(&mut self, tok: &Token) {
        assert!(self.eat(tok))
    }
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
                while self.eat_char(char::is_numeric) {}
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
        match self.peek_char() {
            Some(peek) if f(peek) => {
                self.next_char();
                true
            }
            _ => false,
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
        } else if let Some(c) = self.src.get(self.cur).map(|c| *c as char) {
            (c, self.cur + 1)
        } else {
            return None;
        };

        self.peek_char = Some((peek, idx));
        Some(peek)
    }

    fn peek_char_raw(&self, c: char, n: usize) -> bool {
        self.src
            .get(self.cur + n)
            .map_or(false, |next| c == *next as char)
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

    fn test_eval_impl(expr: &str, var_env: &HashMap<&str, Expr>) {
        let expr = parse(expr);
        assert!(matches!(
            expr.eval(var_env).unwrap(),
            EvaluationResult::Integer(1)
        ));
    }

    fn test_eval_no_var_impl(expr: &str) {
        test_eval_impl(expr, &HashMap::new());
    }

    #[test]
    fn test_eval_no_env() {
        test_eval_no_var_impl("(1 + 2 * 3 - 6) = 1 ");
        test_eval_no_var_impl("(1 + 2 / 3) = 1");
        test_eval_no_var_impl("(10 % 3) = 1");
        test_eval_no_var_impl("(2 * 3 ** 2) = 18");
        test_eval_no_var_impl("(2 ** 3 ** 2) = 512");
        test_eval_no_var_impl("(1 << 2 + 2 >> 1) = 8");
        test_eval_no_var_impl("(1 || 1 && 0) = 1");
        test_eval_no_var_impl("((1 <> 0) + (1 = 1)) = 2");
        test_eval_no_var_impl("((1 > 0) + (1 > 1) + (1 >= 1) + (1 >= 2)) = 2");
        test_eval_no_var_impl("((0 < 1) + (1 < 1) + (1 <= 1) + (2 <= 1)) = 2");
        test_eval_no_var_impl("(0xff00 & 0xf0f0) = 0xf000");
        test_eval_no_var_impl("(0xff00 | 0xf0f0) = 0xfff0");
        test_eval_no_var_impl("(0xff00 ^ 0xf0f0) = 0x0ff0");
        test_eval_no_var_impl("(~0) = (0 - 1)");
    }

    #[test]
    fn test_eval_with_env() {
        let env = vec![
            ("VAR1", Expr::Integer(1)),
            ("EPS", Expr::Float(f64::EPSILON)),
        ]
        .into_iter()
        .collect();

        test_eval_impl("((SIN(PI) - VAR1) < EPS) = 1", &env);
        test_eval_impl("(LN(E) - 1 < EPS) = 1", &env);
    }
}
