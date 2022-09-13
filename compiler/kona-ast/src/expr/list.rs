// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use crate::token::Token;

use super::Expr;

pub struct List {
    pub l_square: Token,
    pub elems: Vec<ListElem>,
    pub r_square: Token,
}

pub struct ListElem {
    pub expr: Box<Expr>,
    pub comma: Token,
}
