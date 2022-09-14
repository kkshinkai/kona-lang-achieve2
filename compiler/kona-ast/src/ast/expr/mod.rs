// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use kona_diagnostic::source::Span;

use super::token::{Literal, Token};

mod ident;
mod record;
mod tuple;
mod list;
mod seq;
mod let_in;

pub use ident::*;
pub use record::*;
pub use tuple::*;
pub use list::*;
pub use seq::*;
pub use let_in::*;

pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

pub enum ExprKind {
    Lit(Literal),
    Ident { op: Option<Token>, ident: LongIdent },
    Record(Record),
    Tuple(Tuple),
    List(List),
    Seq(Seq),
    LetIn(LetIn),
    Paren { l_paren: Token, expr: Box<Expr>, r_paren: Token },
    Pending(Vec<PendingItem>),
    Typed { expr: Box<Expr>, colon: Token, ty: Box<(/* Ty */)> },
    AndAlso { left: Box<Expr>, and_also_kw: Token, right: Box<Expr> },
    OrElse { left: Box<Expr>, or_else_kw: Token, right: Box<Expr> },
    IfElse { if_kw: Token, cond: Box<Expr>, then_kw: Token, then_expr: Box<Expr>, else_kw: Token, else_expr: Box<Expr> },
    While { while_kw: Token, cond: Box<Expr>, do_kw: Token, body: Box<Expr> },
    Case { case_kw: Token, expr: Box<Expr>, of_kw: Token, matches: Matches },
    Fn { fn_kw: Token, matches: Matches },
}

pub enum PendingItem {
    Expr(Box<Expr>),
    Ident(LongIdent),
}

pub struct Matches {
    pub matches: Vec<MatchItem>,
}

pub struct MatchItem {
    pub pat: Box<(/* Pat */)>,
    pub arrow: Token,
    pub expr: Box<Expr>,
}
