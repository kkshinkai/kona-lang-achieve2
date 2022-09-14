// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use super::expr::LongIdent;

pub enum Ty {
    /// Type variable.
    ///
    /// ```text
    /// ty ::= tyvar
    /// ```
    Var(LongIdent),

    /// Record type.
    ///
    /// ```text
    /// ty ::= "{" [ tyrow ] "}"
    /// tyrow ::= lab ":" ty [ "," tyrow ]
    /// ```
    Record,

    Tuple,

    Fn,

    Paren,
}
