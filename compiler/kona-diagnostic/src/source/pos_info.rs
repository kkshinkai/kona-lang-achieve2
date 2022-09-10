// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::fmt;
use std::rc::Rc;

use super::SourceFile;

/// Represents a set of human-readable position information, including the line,
/// column, file name, some other metadata of the source file.
///
/// You can get the [`PosInfo`] with a [`Pos`] in the source manager
/// [`SourceMgr`].
#[derive(Clone, PartialEq, Eq)]
pub struct PosInfo {
    // NOTE: We use `Rc<SourceFile>` here to avoid creating a new string,
    // actually we only need the source file name.

    /// Information about the original source.
    file: Rc<SourceFile>,

    /// The 1-based line number.
    line: usize,

    /// The 1-based column offset.
    col: usize,

    /// The 0-based column offset when displayed.
    col_display: usize,
}

impl PosInfo {
    /// Creates a new [`PosInfo`] with the given file, line, column, and column
    /// offset when displayed.
    pub fn new(file: Rc<SourceFile>, line: usize, col: usize, col_display: usize) -> PosInfo {
        PosInfo { file, line, col, col_display }
    }

    pub fn name(&self) -> String {
        self.file.name()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn col_display(&self) -> usize {
        self.col_display
    }
}

impl fmt::Debug for PosInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: This may be wrong, test it later.
        f.debug_struct("PosInfo")
            .field("file", &self.file.name())
            .field("line", &self.line)
            .field("column", &self.col)
            .finish()
    }
}
