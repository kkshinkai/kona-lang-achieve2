// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use crate::token::Token;

pub struct LongIdent {
    pub op: Option<Token>,
    pub modules: Vec<ModuleIdent>,
    pub ident: Token,
}

pub struct ModuleIdent {
    pub ident: Token,
    pub dot: Token,
}
