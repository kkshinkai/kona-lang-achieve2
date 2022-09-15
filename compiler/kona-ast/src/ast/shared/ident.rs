// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use kona_arena::intern::Symbol;
use kona_diagnostic::source::Span;

use crate::ast::token::Token;

/// A short Kona identifier. It can be a stand-alone identifier, or it can be
/// part of a long identifier (aka. [`QualifiedIdent`]).
///
/// There are four different types of identifiers:
///
/// - Alphanumeric identifiers, such as `hello`, `x`, etc.;
/// - Symbolic identifiers, such as `*`, `++`, etc.;
/// - Type variable, such as `'a`, `'b`, etc.;
/// - Equality type variable, such as `''a`, `''b`, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub ident_tk: Token,
    pub name: Symbol,
    pub kind: IdentKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentKind {
    Symbolic,
    Alphanumeric,
    TyVar,
    EqTyVar,
}

/// A long Kona identifier (aka. qualified identifier).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedIdent {
    pub structs: Vec<MemberAccess>,
    pub ident: Ident,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberAccess {
    pub name: Ident,
    pub dot_tk: Token,
}
