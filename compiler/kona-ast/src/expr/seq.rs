// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use crate::token::Token;

use super::Expr;

pub struct Seq {
    pub l_paren: Token,
    pub exprs: Vec<SeqItem>,
    pub r_paren: Token,
}

pub struct SeqItem {
    pub expr: Box<Expr>,
    pub semi: Token,
}
