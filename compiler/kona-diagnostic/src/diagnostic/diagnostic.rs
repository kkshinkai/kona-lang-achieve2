// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use crate::source::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub labels: DiagnosticLabels,
}

impl Diagnostic {
    pub fn span(&self) -> Span {
        self.labels.primary_label.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Level {
    /// A compilation error.
    ///
    /// Currently Kona has no error recovery mechanism, all errors are
    /// considered fatal. It always panic the program when emitting.
    Error,

    /// A warning or lint.
    Warn,

    /// An informational message during compilation.
    ///
    /// Unlike "note" diagnostics in most other compilers (e.g. Clang), here we
    /// use it as a log. This practice will be discarded in the future.
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticLabels {
    pub primary_label: DiagnosticLabel,
    pub sublabels: Vec<DiagnosticLabel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticLabel {
    pub span: Span,
    pub message: String,
}
