// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::path::PathBuf;

/// Represents the path to a source file. Can only be created by [`SourceMap`].
///
/// This path, unlike [`PathBuf`] and [`Path`], represents a file that can be
/// read without problems.
///
/// We should not provide the method to create the [`SourcePath`] publicly. All
/// source paths should be generated by [`SourceMap`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourcePath {
    kind: SourcePathKind,
}

impl SourcePath {
    pub(super) fn local_file(path: PathBuf) -> SourcePath {
        SourcePath {
            kind: SourcePathKind::Local(path)
        }
    }

    pub(super) fn test_file(name: Option<String>, uid: u32) -> SourcePath {
        SourcePath {
            kind: SourcePathKind::Test { name, uid }
        }
    }

    pub(super) fn is_local_file(&self) -> bool {
        matches!(self.kind, SourcePathKind::Local(_))
    }

    pub(super) fn is_test_file(&self) -> bool {
        matches!(self.kind, SourcePathKind::Test { .. })
    }

    pub(super) fn readable_name(&self) -> String {
        use path_helper::{clear_unc_prefix, diff_paths};
        match self.kind {
            SourcePathKind::Local(ref path) => {
                if let Ok(cwd) = std::env::current_dir() {
                    if let Some(relative) = diff_paths(path, cwd) {
                        return relative.to_string_lossy().to_string();
                    }
                }
                clear_unc_prefix(path).to_string_lossy().to_string()
            },
            SourcePathKind::Test { ref name, uid } => match name {
                Some(name) => name.clone(),
                None => format!("virtual #{}", uid),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum SourcePathKind {
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

mod path_helper {
    use std::path::{PathBuf, Path, Component, Prefix};

    /// Get relative path from base to the given path.
    pub fn diff_paths<P, B>(path: P, base: B) -> Option<PathBuf>
    where
        P: AsRef<Path>,
        B: AsRef<Path>,
    {
        let path = path.as_ref();
        let base = base.as_ref();

        if path.is_absolute() != base.is_absolute() {
            if path.is_absolute() {
                Some(PathBuf::from(path))
            } else {
                None
            }
        } else {
            let mut ita = path.components();
            let mut itb = base.components();
            let mut comps: Vec<Component> = vec![];
            loop {
                match (ita.next(), itb.next()) {
                    (None, None) => break,
                    (Some(a), None) => {
                        comps.push(a);
                        comps.extend(ita.by_ref());
                        break;
                    }
                    (None, _) => comps.push(Component::ParentDir),
                    (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                    (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                    (Some(_), Some(b)) if b == Component::ParentDir => return None,
                    (Some(a), Some(_)) => {
                        comps.push(Component::ParentDir);
                        for _ in itb {
                            comps.push(Component::ParentDir);
                        }
                        comps.push(a);
                        comps.extend(ita.by_ref());
                        break;
                    }
                }
            }
            Some(comps.iter().map(|c| c.as_os_str()).collect())
        }
    }

    // TBD: Maybe we should use `std::path::absolute` (unstable) instead of
    // `std::fs::canonicalize` in our compiler?

    /// Clear the indows NT UNC path prefix (e.g. `\\?\` in
    /// `\\?\C:\example.txt`).
    ///
    /// Rust `std::fs::canonicalize` returns Windows NT UNC paths on Windows
    /// (e.g. `\\?\C:\example.txt`), which are rarely supported by Windows
    /// programs, even Microsoft's own. To avoid confusing users, sometimes we
    /// should remove this prefix.
    ///
    /// `clear_unc_prefix` expects the input path to already be canonicalized,
    /// but it is not a hard requirement.
    pub fn clear_unc_prefix<P>(path: P) -> PathBuf where P: AsRef<Path> {
        let path = path.as_ref();

        #[cfg(windows)]
        {
            if let Some(Component::Prefix(p)) = path.components().next() {
                // Checks for verbatim disk prefixes (e.g `\\?\C:` or `C:`).
                if matches!(p.kind(), Prefix::VerbatimDisk(..)) {
                    return path.to_str()
                        .and_then(|s| if s.starts_with(r"\\?\") {
                            // Trims the verbatim prefix if the path string is
                            // started with `\\?\`. This is actually not very
                            // rigorous, but since we use this function almost
                            // exclusively for paths that are canonicalized by
                            // `std::fs::canonicalize`, it's not too buggy.
                            s.get(4..)
                        } else {
                            Some(s)
                        })
                        .map(PathBuf::from)
                        .unwrap_or_else(|| path.to_path_buf())
                }
            }
        }

        path.to_path_buf()
    }
}

#[cfg(test)]
mod path_helper_tests {
    use std::path::PathBuf;

    use super::path_helper::{clear_unc_prefix, diff_paths};

    #[test]
    fn test_diff_paths() {
        assert_eq!(diff_paths("/foo/bar", "/foo/bar/baz"),
            Some(PathBuf::from("../")));
        assert_eq!(diff_paths("~/project/src/compiler/main.sml", "~/project"),
            Some(PathBuf::from("src/compiler/main.sml")));
    }

    #[test]
    fn test_clear_unc_prefix() {
        #[cfg(windows)]
        {
            assert_eq!(clear_unc_prefix(""), PathBuf::from(""));
            assert_eq!(clear_unc_prefix(r"\\?\C:\example.txt"),
                PathBuf::from(r"C:\example.txt"));
            assert_eq!(clear_unc_prefix(r"C:\example.txt"),
                PathBuf::from(r"C:\example.txt"));
        }

        assert_eq!(clear_unc_prefix(""), PathBuf::from(""));
        assert_eq!(clear_unc_prefix(r"~/example.txt"),
            PathBuf::from(r"~/example.txt"));
        assert_eq!(clear_unc_prefix(r"/usr/kkshinkai/example.txt"),
            PathBuf::from(r"/usr/kkshinkai/example.txt"));
    }
}
