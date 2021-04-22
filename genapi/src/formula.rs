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
