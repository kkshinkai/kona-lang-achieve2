// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use crate::ast::token::Token;

use super::Expr;

pub struct Tuple {
    pub l_paren: Token,
    pub elems: Vec<TupleElem>,
    pub r_paren: Token,
}

pub struct TupleElem {
    pub expr: Box<Expr>,
    pub comma: Token,
}
