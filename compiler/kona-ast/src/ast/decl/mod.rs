// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use kona_diagnostic::source::Span;

pub struct Decl {
    pub kind: DeclKind,
    pub span: Span,
}

pub enum DeclKind {

}
