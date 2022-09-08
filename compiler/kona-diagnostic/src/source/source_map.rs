// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::{rc::Rc, collections::HashMap, path::PathBuf, io, fs, sync::{RwLock, atomic::{AtomicUsize, Ordering, AtomicU32}}};

use crate::source::SourceFile;

use super::{Pos, SourcePath, Span, PosInfo, SourceLine};

// FIXME: Try load an empty file.

/// Source map for a compilation unit, including a bunch of source files, source
/// code, and position information.
///
/// [`SourceMap`] is the top-level interface for source code management. It
/// manages a collection of [`SourceFile`]s and the source code within them.
/// [`SourceMap`] provides a position assignment mechanism that allocates a
/// unique position [`Pos`] for each byte in the source code. You can get a
/// human-readable information [`PosInfo`] with a [`Pos`], or read a span of
/// source code with a [`Range<Pos>`] in source map.
pub struct SourceMap {
    /// The used position index, for allocating individual position intervals to
    /// source files.
    used_pos_space: AtomicUsize,

    /// The used virtual file number.
    used_virtual_file_number: AtomicU32,

    // WARNING: Don't modify `used_pos_space` directly. Don't add new functions
    // that might modify or access it. `allocate_pos_space` should be the
    // only function that can increase `used_pos_space`.

    source_files: RwLock<SourceMapFiles>,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap {
            // Position 0 is reserved for the dummy span.
            used_pos_space: AtomicUsize::new(1),
            used_virtual_file_number: AtomicU32::new(0),
            source_files: RwLock::new(SourceMapFiles::default()),
        }
    }

    fn allocate_pos_space(&self, size: usize) -> usize {
        let current = self.used_pos_space.load(Ordering::Relaxed);

        loop {
            let next = current
                .checked_add(size)
                // Add some space between files to help us distinguish the
                // zero-length files.
                .and_then(|next| next.checked_add(1))
                .expect("unable to allocate more space for source code");

            if self
                .used_pos_space
                .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return usize::try_from(current).unwrap();
            }
        }
    }

    fn allocate_virtual_file_number(&self) -> u32 {
        let current = self.used_virtual_file_number.load(Ordering::Relaxed);

        loop {
            let next = current
                .checked_add(1)
                .expect("unable to allocate more space for source code");

            if self
                .used_virtual_file_number
                .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return current;
            }
        }
    }

    /// Loads source file from the given path.
    pub fn load_local_file(
        &self, path: PathBuf,
    ) -> io::Result<Rc<SourceFile>> {
        // Path must be absolute to uniquely identify the source file.
        let file_path = SourcePath::local_file(fs::canonicalize(&path)?);

        {
            let files = self.source_files.read().unwrap();
            if let Some(sf) = files.files_map.get(&file_path) {
                return Ok(sf.clone());
           }
        }

        // FIXME: Just don't read and canonicalize this file twice.
        let src = fs::read_to_string(&path)?;
        let start_pos = Pos::from_usize(self.allocate_pos_space(src.len()));
        let file = Rc::new(
            SourceFile::local_file(fs::canonicalize(&path)?, start_pos)?
        );

        {
            let mut files = self.source_files.write().unwrap();
            files.files.push(file.clone());
            files.files_map.insert(file_path, file.clone());
        }

        Ok(file)
    }

    /// Adds a test source file with the given name and source string.
    pub fn load_test_file(
        &self, name: Option<String>, src: String
    ) -> Rc<SourceFile> {
        let uid = self.allocate_virtual_file_number();
        let file_path = SourcePath::test_file(name.clone(), uid);

        {
            let files = self.source_files.read().unwrap();
            if let Some(sf) = files.files_map.get(&file_path) {
                return sf.clone();
           }
        }

        let start_pos = Pos::from_usize(self.allocate_pos_space(src.len()));
        let file = Rc::new(
            SourceFile::test_file(Rc::new(src), name, uid, start_pos)
        );

        {
            let mut files = self.source_files.write().unwrap();
            files.files.push(file.clone());
            files.files_map.insert(file_path, file.clone());
        }

        file
    }

    /// Creates a single-file source map, mostly for testing.
    pub fn from_file(path: PathBuf) -> io::Result<Self> {
        let source_map = SourceMap::new();
        source_map.load_local_file(path)?;
        Ok(source_map)
    }

    /// Creates a source map from the given source string, mostly for testing.
    pub fn from_string(src: impl Into<String>) -> Self {
        let source_map = SourceMap::new();
        source_map.load_test_file(Some("<string>".to_string()), src.into());
        source_map
    }
}

pub type LookupResult<T> = Result<T, LookupError>;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupError {
    /// Tried to get source information from a non-existent position.
    DummyPosOrSpan,

    /// Tried to get source information from a non-existent span.
    DummySpan,

    // TODO: We need clearer documentation to distinguish between
    // `PosOutOfRange`, `SpanOutOfRange` and `SpanAcrossFiles`.

    /// Cannot find the source file that covers the given position or span.
    OutOfRange,

    /// Both start and end positions of the span are legal, but exist in
    /// different source files. In most cases this is meaningless.
    SpanAcrossFiles,
}

impl SourceMap {
    pub fn contains_pos(&self, pos: Pos) -> LookupResult<()> {
        self.lookup_file_at_pos(pos).map(|_| ())
    }

    pub fn contains_span(&self, span: Span) -> LookupResult<()> {
        self.lookup_file_at_span(span).map(|_| ())
    }

    pub fn lookup_pos_info(&self, pos: Pos) -> LookupResult<PosInfo> {
        let file = self.lookup_file_at_pos(pos)?;
        let (line, col, col_display) = file.lookup_line_col_and_col_display(pos);
        Ok(PosInfo::new(file, line, col, col_display))
    }

    /// Finds the source file containing the given position.
    pub fn lookup_file_at_pos(&self, pos: Pos) -> LookupResult<Rc<SourceFile>> {
        if pos.is_dummy() {
            return Err(LookupError::DummyPosOrSpan);
        }

        let files = self.source_files.read().unwrap();
        let search_result = files.files
            .binary_search_by_key(&pos, |file| file.start_pos());

        match search_result {
            Ok(idx) => {
                let file = &files.files[idx];
                if file.is_empty() {
                    Err(LookupError::OutOfRange)
                } else {
                    Ok(file.clone())
                }
            },
            Err(res) => {
                // The start position of the first file must be 1. We've checked
                // that pos is not dummy (aka. >= 1), so the search result must
                // be greater than 0.
                debug_assert!(res > 0 && res <= files.files.len());
                let idx = res - 1;

                // The position must be in the range of the file.
                let file = &files.files[idx];
                if file.contains_pos(pos) {
                    Ok(file.clone())
                } else {
                    Err(LookupError::OutOfRange)
                }
            },
        }
    }

    /// Finds the source file containing the given span.
    pub fn lookup_file_at_span(&self, span: Span) -> LookupResult<Rc<SourceFile>> {
        let start_file = self.lookup_file_at_pos(span.start())?;
        let end_file = self.lookup_file_at_pos(span.end() - 1u32)?;

        // FIXME: For now we use the span to check the equality of two files.
        // This is really a hacky way. Maybe we should implement file ID later.
        if start_file.span() == end_file.span() {
            Ok(start_file)
        } else {
            Err(LookupError::SpanAcrossFiles)
        }
    }

    pub fn lookup_line_at_pos(&self, pos: Pos) -> LookupResult<SourceLine> {
        let file = self.lookup_file_at_pos(pos)?;
        let line = file.lookup_line_at_pos(pos).unwrap() as u32;
        Ok(SourceLine::new(file, line))
    }

    pub fn lookup_lines_at_span(&self, span: Span) -> LookupResult<Vec<SourceLine>> {
        let start_file = self.lookup_file_at_pos(span.start())?;
        let end_file = self.lookup_file_at_pos(span.end() - 1u32)?;

        // FIXME: For now we use the span to check the equality of two files.
        // This is really a hacky way. Maybe we should implement file ID later.
        if start_file.span() == end_file.span() {
            let start_line = start_file.lookup_line_at_pos(span.start()).unwrap();
            let end_line = end_file.lookup_line_at_pos(span.end() - 1u32).unwrap();

            Ok((start_line..=end_line)
                .map(|line| SourceLine::new(start_file.clone(), line as u32))
                .collect())
        } else {
            Err(LookupError::SpanAcrossFiles)
        }
    }

    /// Returns the source file at the given span.
    pub fn lookup_source(&self, span: Span) -> LookupResult<String> {
        // TBD: We should return `&str` instead of creating a new `String`,
        // because the source file is immutable. We haven't figured out how to
        // struggle with ownership, but we should fix it if possible.

        let file = self.lookup_file_at_span(span)?;

        let start = span.start().to_usize() - file.start_pos().to_usize();
        let end = span.end().to_usize() - file.start_pos().to_usize();

        Ok(file.src()[start..end].to_string())
    }
}

#[derive(Default)]
pub struct SourceMapFiles {
    /// The source files.
    files: Vec<Rc<SourceFile>>,

    /// The source files hash map.
    files_map: HashMap<SourcePath, Rc<SourceFile>>,
}
