// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::{path::PathBuf, rc::Rc, ops::Range, io, fs};

use unicode_width::UnicodeWidthChar;

use super::{Span, Pos};

#[derive(Clone, PartialEq, Eq)]
pub struct SourceFile {
    path: SourcePath,

    src: Rc<String>,

    /// Source span of the file.
    ///
    /// Each file is assigned a unique index range.
    span: Span,

    /// Caches the start of each line in the source file.
    ///
    /// Line breaks include carriage return (CR, `\r`), line feed (LF, `\n`), and
    /// carriage return followed by line feed (CRLF, `\r\n`). These three kinds
    /// of line breaks can be mixed in the same file (although this is a bad
    /// practice).
    lines: Vec<Pos>,

    /// Caches the position of all multi-byte characters in the source file.
    ///
    /// TODO: More explanation for UTF-8 encoding.
    multi_byte_chars: Vec<MultiByteChar>,

    /// Caches the position of characters that are not narrow in the source
    /// file.
    ///
    /// This property may be used when printing source code and error messages
    /// in the terminal. See also Unicode Standard Annex #11 [East Asian Width].
    ///
    /// [East Asian Width]: https://www.unicode.org/reports/tr11/
    non_narrow_chars: Vec<NonNarrowChar>,
}

impl SourceFile {
    /// Creates and reads the source file from the given path.
    ///
    /// The given path must be canonicalized. Actually, before creating this
    /// `SourceFile`, we have usually already checked the size of the file.
    /// We should have already canonicalized the path and checked if it exists.
    /// The overhead of accessing the filesystem should not be repaid in this
    /// function.
    pub fn local_file(path: PathBuf, start_pos: Pos) -> io::Result<SourceFile> {
        debug_assert!(matches!(path.canonicalize(), Ok(p) if p == path),
            "path for `SourceFile::local_file` must be canonicalized");

        let src = fs::read_to_string(&path)?;

        Ok(SourceFile::new(SourcePath::Local(path), Rc::new(src), start_pos))
    }

    /// Creates a virtual testing source file from the given source.
    pub fn test_file(src: Rc<String>, name: Option<String>, start_pos: Pos) -> SourceFile {
        SourceFile::new(SourcePath::Test {
            name,
            uid: start_pos.to_u32(),
         }, src, start_pos)
    }

    /// Creates a new source file from the given path and source code.
    fn new(path: SourcePath, src: Rc<String>, start_pos: Pos) -> SourceFile {
        let end_pos = start_pos + src.len();
        let (lines, multi_byte_chars, non_narrow_chars) =
            SourceFile::analyze(&src, start_pos);
        SourceFile {
            src,
            path,
            span: Span::new(start_pos, end_pos),
            lines,
            multi_byte_chars,
            non_narrow_chars,
        }
    }

    /// Gets the human-readable file name of the source file for diagnostic
    /// messages. Note his name cannot be used as a path.
    pub fn name(&self) -> String {
        match self.path {
            SourcePath::Local(ref path) => {
                // Rust `std::fs::canonicalize` returns Windows NT UNC paths on
                // Windows (e.g. `\\?\C:\example.txt`), which are rarely
                // supported by Windows programs, even Microsoft's own. Just
                // remove the verbatim prefix.
                //
                // This path is already canonicalized, so we don't need to
                // verify it again.
                //
                // TBD: Maybe we should use `std::path::absolute` (unstable)
                // instead of `std::fs::canonicalize`?
                #[cfg(windows)]
                {
                    use std::path::{Component, Prefix};
                    if let Some(Component::Prefix(p)) = path.components().next() {
                        if matches!(p.kind(), Prefix::VerbatimDisk(..)) {
                            // This string if always longer than 4 on Windows
                            // because it is canonicalized.
                            return path.to_string_lossy()[4..].to_string();
                        }
                    }
                }

                path.to_string_lossy().to_string()
            },
            SourcePath::Test { ref name, uid } => match name {
                Some(name) => name.clone(),
                None => format!("virtual #{}", uid),
            },
        }
    }

    pub fn is_local_file(&self) -> bool {
        matches!(self.path, SourcePath::Local(_))
    }

    pub fn is_test_file(&self) -> bool {
        matches!(&self.path, SourcePath::Test { .. })
    }

    /// Finds the line containing the given position.
    ///
    /// The return value is the index into the `lines` array of this
    /// `SourceFile`, not the 1-based line number. If the source file is empty
    /// or the position is located before the first line, `None` is returned.
    pub fn lookup_line(&self, pos: Pos) -> Option<usize> {
        match self.lines.binary_search(&pos) {
            Ok(index) => Some(index),
            Err(0) => None,
            Err(index) => Some(index - 1),
        }
    }

    pub fn lookup_line_bounds(&self, line_index: usize) -> Range<Pos> {
        assert!(line_index < self.lines.len());

        if self.is_empty() {
            self.span.start()..self.span.end()
        } else if line_index == (self.lines.len() - 1) {
            self.lines[line_index]..self.span.end()
        } else {
            self.lines[line_index]..self.lines[line_index + 1]
        }
    }


    /// Looks up the file's 1-based line number and 0-based column offset, for a
    /// given [`Pos`].
    pub fn lookup_line_and_col(&self, pos: Pos) -> (usize, usize) {
        if let Some(line) = self.lookup_line(pos) {
            let line_start = self.lines[line];
            let col = {
                let linebpos = self.lines[line];
                let start_idx = self.multi_byte_chars
                    .binary_search_by_key(&linebpos, |x| x.pos())
                    .unwrap_or_else(|x| x);
                let extra_byte = self
                    .multi_byte_chars[start_idx..]
                    .iter()
                    .take_while(|x| x.pos() < pos)
                    .map(|x| x.len() as usize - 1)
                    .sum::<usize>();
                pos.to_usize() - line_start.to_usize() - extra_byte
            };
            (line + 1, col)
        } else {
            (0, 0)
        }
    }

    pub fn lookup_line_col_and_col_display(
        &self, pos: Pos
    ) -> (usize, usize, usize) {
        let (line, col) = self.lookup_line_and_col(pos);
        let col_display = {
            let linebpos = self.lines[line - 1];
            let start_idx = self
                .non_narrow_chars
                .binary_search_by_key(&linebpos, |x| x.pos())
                .unwrap_or_else(|x| x);
            let non_narrow = self
                .non_narrow_chars[start_idx..]
                .iter()
                .take_while(|x| x.pos() < pos);
            let width = non_narrow.clone()
                .map(|x| x.width())
                .sum::<usize>();
            let count = non_narrow.count();
            col + width - count
        };
        (line, col, col_display)
    }

    #[inline]
    pub fn contains(&self, pos: Pos) -> bool {
        pos >= self.span.start() && pos <= self.span.end()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.span.start() == self.span.end()
    }

    /// Finds all newlines, multi-byte characters, and non-narrow characters in a
    /// source file.
    fn analyze(
        src: &str,
        start_pos: Pos,
    ) -> (Vec<Pos>, Vec<MultiByteChar>, Vec<NonNarrowChar>) {
        let mut lines = vec![start_pos];
        let mut multi_byte_chars = vec![];
        let mut non_narrow_chars = vec![];

        let offset = start_pos.to_usize();

        let mut idx = 0;
        let src_bytes = src.as_bytes();

        while idx < src_bytes.len() {
            let byte = src_bytes[idx];

            // How much to advance in order to get to the next UTF-8 char in the
            // string.
            let mut char_len = 1;

            if byte < 32 {
                // This is an ASCII control character.
                let pos = Pos::from_usize(idx + offset);

                match byte {
                    b'\n' => lines.push(pos + 1u32),
                    b'\r' if src_bytes.get(idx + 1) != Some(&b'\n') => {
                        lines.push(pos + 1u32);
                    },
                    b'\t' => non_narrow_chars.push(NonNarrowChar::new(pos, 4)),
                    _ => non_narrow_chars.push(NonNarrowChar::new(pos, 0)),
                }
            } else if byte >= 127 {
                // This is either ASCII control character "DEL" or the beginning of
                // a multibyte char. Just decode to `char`.
                let char = (&src[idx..]).chars().next().unwrap();
                char_len = char.len_utf8();

                let pos = Pos::from_usize(idx + offset);

                if char_len > 1 {
                    multi_byte_chars.push(MultiByteChar::new(pos, char_len as u8));
                }

                let char_width = UnicodeWidthChar::width(char).unwrap_or(0);

                if char_width != 1 {
                    non_narrow_chars.push(NonNarrowChar::new(pos, char_width));
                }
            }

            idx += char_len;
        }

        // The code above optimistically registers a new line after each newline
        // it encounters. If that point is already outside the source file, remove
        // it again.
        if let Some(&last_line_start) = lines.last() {
            let end_pos = Pos::from_usize(src.len() + offset);
            if last_line_start == end_pos {
                lines.pop();
            }
        }

        (lines, multi_byte_chars, non_narrow_chars)
    }
}

#[derive(Clone, PartialEq, Eq)]
enum SourcePath {
    /// The canonical, unique path to an existing local file. The path must be
    /// canonicalized by [`std::fs::canonicalize`].
    Local(PathBuf),

    /// A dummy file with given name, mostly for testing.
    Test {
        /// An optional name for the testing source snippet.
        name: Option<String>,

        /// An unique number to distinguish the testing snippet from others.
        uid: u32,
    },
}

/// Represents a multi-byte UTF-8 unicode scalar in the source code.
#[derive(Clone, Debug, PartialEq, Eq)]
struct MultiByteChar {
    pos: Pos,

    /// The number of bytes in the UTF-8 encoding of the character. It could
    /// only be 2, 3 or 4.
    len: u8,
}

impl MultiByteChar {
    /// Creates a new [`MultiByteChar`] from [`Pos`] and its length.
    #[inline]
    fn new(pos: Pos, len: u8) -> Self {
        MultiByteChar { pos, len }
    }

    /// Returns the UTF-8 length of this character.
    #[inline]
    fn len(&self) -> u8 {
        self.len
    }

    /// Returns the [`Pos`] of this character.
    #[inline]
    fn pos(&self) -> Pos {
        self.pos
    }
}

/// Represents a non-narrow character in the source code.
#[derive(Clone, Debug, PartialEq, Eq)]
struct NonNarrowChar {
    pos: Pos,
    kind: NonNarrowCharKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NonNarrowCharKind {
    /// A zero-width character.
    ZeroWidth,

    /// A full-width character.
    Wide,

    /// A tab, treated as four spaces.
    Tab,
}

impl NonNarrowChar {
    /// Creates a new [`NonNarrowChar`] from [`Pos`] and its east asian
    /// width.
    fn new(pos: Pos, width: usize) -> Self {
        let kind = match width {
            0 => NonNarrowCharKind::ZeroWidth,
            2 => NonNarrowCharKind::Wide,
            _ => NonNarrowCharKind::Tab,
        };
        NonNarrowChar { pos, kind }
    }

    /// Returns the position of this character.
    #[inline]
    fn pos(&self) -> Pos {
        self.pos
    }

    /// Returns the width of this character.
    fn width(&self) -> usize {
        match self.kind {
            NonNarrowCharKind::ZeroWidth => 0,
            NonNarrowCharKind::Wide => 2,
            NonNarrowCharKind::Tab => 4,
        }
    }
}
