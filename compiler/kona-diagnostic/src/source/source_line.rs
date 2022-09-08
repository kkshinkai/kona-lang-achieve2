// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::{rc::Rc, fmt};

use super::{SourceFile, Span};

#[derive(Clone, PartialEq, Eq)]
pub struct SourceLine {
    file: Rc<SourceFile>,
    line: u32,
}

impl SourceLine {
    pub fn new(file: Rc<SourceFile>, line: u32) -> Self {
        SourceLine { file, line }
    }

    pub fn line_number(&self) -> u32 {
        self.line + 1
    }

    pub fn source(&self) -> String {
        self.file.lookup_line_source(self.line as usize)
    }

    pub fn span(&self) -> Span {
        self.file.lookup_line_span(self.line as usize)
    }
}

impl fmt::Debug for SourceLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SourceLine")
            .field("file", &self.file.name())
            .field("line (0-based)", &self.line)
            .finish()
    }
}
