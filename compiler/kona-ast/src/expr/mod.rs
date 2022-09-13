// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use kona_diagnostic::source::Span;

use crate::token::{Literal, Token};

mod ident;
mod record;
mod tuple;
mod list;
mod seq;

pub use ident::*;
pub use record::*;
pub use tuple::*;
pub use list::*;
pub use seq::*;

pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

pub enum ExprKind {
    Lit(Literal),
    Ident(LongIdent),
    Record(Record),
    Tuple(Tuple),
    List(List),
    Seq(Seq),
}
