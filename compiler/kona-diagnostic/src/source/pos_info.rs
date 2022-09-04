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
    /// Information about the original source.
    pub file: Rc<SourceFile>, // TBD: we don't need all these information, why
                              // not just `FilePath`?

    /// The 1-based line number.
    pub line: usize,

    /// The 0-based column offset.
    pub col: usize,

    /// The 0-based column offset when displayed.
    pub col_display: usize,
}

impl PosInfo {
    /// Creates a new [`PosInfo`] with the given file, line, column, and column
    /// offset when displayed.
    pub fn new(file: Rc<SourceFile>, line: usize, col: usize, col_display: usize) -> PosInfo {
        PosInfo { file, line, col, col_display }
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
