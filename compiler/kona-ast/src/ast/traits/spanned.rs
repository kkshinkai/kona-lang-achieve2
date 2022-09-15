// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use kona_diagnostic::source::Span;

pub trait Spanned {
    fn span(&self) -> Span;
}

#[cfg(test)]
mod tests {
    use kona_diagnostic::source::Span;
    use kona_proc_macros::Spanned;
    use crate::ast::Spanned;

    #[derive(Debug, PartialEq, Eq, Spanned)]
    pub struct Node {
        span: Span,
    }

    #[test]
    fn has_span() {
        let node = Node { span: Span::dummy() };
        assert_eq!(node.span(), Span::dummy());
    }
}
