// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::rc::Rc;

use super::SourceFile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLine {
    file: Rc<SourceFile>,
    line: u32,
}

impl SourceLine {
    pub fn new(file: Rc<SourceFile>, line: u32) -> Self {
        debug_assert!(file.contains_line(line));

        SourceLine { file, line }
    }

    pub fn line_number(&self) -> u32 {
        self.line
    }
}
