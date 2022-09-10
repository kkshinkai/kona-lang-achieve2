// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

mod literal;

pub use literal::*;

use kona_diagnostic::source::Span;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    // Core keywords:
    AbsType,
    And,
    AndAlso,
    As,
    Case,
    DataType,
    Do,
    Else,
    End,
    Exception,
    Fn,
    Fun,
    Handle,
    If,
    In,
    Infix,
    InfixR,
    Let,
    Local,
    NonFix,
    Of,
    Op,
    Open,
    OrElse,
    Raise,
    Rec,
    Then,
    Type,
    Val,
    With,
    WithType,
    While,

    /// `(`.
    LParen,

    /// `)`.
    RParen,

    /// `[`.
    LSquare,

    /// `]`.
    RSquare,

    /// `{`.
    LCurly,

    /// `}`.
    RCurly,

    /// `,`.
    Comma,

    /// `:`.
    Colon,

    /// `;`.
    Semi,

    /// `...`.
    Ellipsis,

    /// `_`.
    Wildcard,

    /// `|`.
    Bar,

    /// `=`.
    Eq,

    /// `=>`.
    DArrow,

    /// `->`.
    Arrow,

    /// `#`.
    NumSign,

    Lit(Literal),

    Comment,

    /// An identifier. It could be an atomic name (e.g. `hello`) or a qualified
    /// name (e.g. `List.length`).
    Ident {
        /// If this identifier is qualified (aka. has dots in it).
        qualified: bool,
    }
}
