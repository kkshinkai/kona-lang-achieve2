// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use crate::token::Token;

use super::LongIdent;

pub struct Record {
    pub l_curly: Token,
    pub fields: Vec<RecordRow>,
    pub r_curly: Token,
}

pub struct RecordRow {
    pub label: Label,
    pub comma: Token,
}

pub enum Label {
    Name(LongIdent),
    Num(u32),
}
