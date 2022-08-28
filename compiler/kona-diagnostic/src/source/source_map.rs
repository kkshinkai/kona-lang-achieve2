// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::{rc::Rc, collections::HashMap, path::PathBuf, io, fs, sync::{RwLock, atomic::{AtomicUsize, Ordering, AtomicU32}}};

use crate::source::SourceFile;

use super::{Pos, SourcePath, Span};

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
        let file_path = fs::canonicalize(&path)?;

        {
            let files = self.source_files.read().unwrap();
            if let Some(sf) = files.files_map.get(&file_path) {
                return Ok(sf.clone());
           }
        }

        let start_pos = Pos::from_usize(self.allocate_pos_space(src.len()));
        let file = Rc::new(
            SourceFile::local_file(file_path.clone(), start_pos)?
        );

        {
            let mut files = self.source_files.write().unwrap();
            files.files.push(file.clone());
            files.files_map.insert(file_path, file.clone());
        }

        Ok(file)
    }

    /// Adds a virtual source file with the given name and source string.
    pub fn load_virtual_file(
        &self, name: Option<String>, src: String
    ) -> Rc<SourceFile> {
        let file_path = SourcePath::Test {
            name,
            uid: self.allocate_virtual_file_number(),
        };

        {
            let files = self.source_files.read().unwrap();
            if let Some(sf) = files.files_map.get(&file_path) {
                return sf.clone();
           }
        }

        let start_pos = Pos::from_usize(self.allocate_pos_space(src.len()));
        let file = Rc::new(
            SourceFile::
            new(file_path.clone(), Rc::new(src), start_pos)
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
        source_map.load_file(path)?;
        Ok(source_map)
    }

    /// Creates a source map from the given source string, mostly for testing.
    pub fn from_string(src: impl Into<String>) -> Self {
        let source_map = SourceMap::new();
        source_map.load_virtual_file("<string>".to_string(), src.into());
        source_map
    }

    pub fn lookup_pos_info(&self, pos: Pos) -> PosInfo {
        let sf = self.lookup_file(pos);
        let (line, col, col_display) = sf.lookup_line_col_and_col_display(pos);
        PosInfo::new(sf, line, col, col_display)
    }

    /// Finds the source file containing the given position.
    pub fn lookup_file(&self, pos: Pos) -> Rc<SourceFile> {
        let files = self.source_files.read().unwrap();
        let idx = files.files
            .binary_search_by_key(&pos, |file| file.start_pos)
            .unwrap_or_else(|p| p - 1);
        files.files[idx].clone()
    }

    /// Returns the source file at the given interval.
    pub fn lookup_source(&self, span: Span) -> String {
        let file = self.lookup_file(span.start);

        let start = span.start.to_usize() - file.start_pos.to_usize();
        let end = span.end.to_usize() - file.start_pos.to_usize();

        file.src[start..end].to_string()
    }
}

#[derive(Default)]
pub struct SourceMapFiles {
    /// The source files.
    files: Vec<Rc<SourceFile>>,

    /// The source files hash map.
    files_map: HashMap<SourcePath, Rc<SourceFile>>,
}
