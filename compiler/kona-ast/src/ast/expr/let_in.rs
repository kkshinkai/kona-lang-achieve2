// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use crate::ast::token::Token;

use super::SeqItem;

pub struct LetIn {
    pub let_kw: Token,
    pub decls: Box<(/* Dec */)>,
    pub in_kw: Token,
    pub exprs: Vec<SeqItem>,
    pub end_kw: Token,
}
