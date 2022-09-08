// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::{rc::Rc, io};

use crate::source::SourceMap;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::diagnostic::{Diagnostic, Level};

// TODO: Two emitters should be able to be composable into another emitter.

pub trait Emitter {
    /// Emit a diagnostic.
    fn emit_diagnostic(&mut self, diag: &Diagnostic);

    // TBD: Why we put `SourceMap` in `Emitter`, not `DiagnosticEngine`?

    /// Returns the `SourceMap` associated with this emitter if any.
    ///
    /// Not all emitters need a source map, for example, a JSON emitter can
    /// just report the position index in the span, it doesn't need to know
    /// the actual source code there.
    fn source_map(&self) -> Option<Rc<SourceMap>>;
}

pub struct TtyEmitter {
    out: StandardStream,
    source_map: Option<Rc<SourceMap>>,
}

impl TtyEmitter {
    pub fn new(source_map: Rc<SourceMap>) -> TtyEmitter {
        TtyEmitter {
            out: StandardStream::stdout(ColorChoice::Auto),
            source_map: Some(source_map),
        }
    }

    pub fn no_source_map() -> TtyEmitter {
        TtyEmitter {
            out: StandardStream::stdout(ColorChoice::Auto),
            source_map: None,
        }
    }
}

impl TtyEmitter {
    fn try_emit_diagnostic(&mut self, diag: &Diagnostic) -> io::Result<()> {
        // Prints colored "error", "warning", or "note".

        let (level_name, level_color, level_mark_char) = match diag.level {
            Level::Error => ("error", Color::Red, '^'),
            Level::Warn => ("warning", Color::Yellow, '^'),
            Level::Note => ("note", Color::Blue, '-'),
        };

        self.with_color(level_color, true, |out| {
            write!(out, "{}", level_name)
        })?;

        writeln!(self.out, ": {}", diag.message)?;

        // Prints the source location if available.
        if let Some(source_map) = self.source_map() {
            if !diag.span().is_dummy() {
                let (start_pos_info, end_pos_info) =
                    (source_map.lookup_pos_info(diag.span().start()),
                     source_map.lookup_pos_info(diag.span().end()));

                // If this span crosses multiple files, just abort.
                //
                // TBD: Should we panic here? How should we handle a span that
                // crosses multiple files?
                if start_pos_info.file != end_pos_info.file {
                    return Ok(());
                }

                let indent = end_pos_info.line.to_string().len();

                // Prints "  --> src/main.sml:1:1"
                self.with_color(Color::Cyan, true, |out| {
                    write!(out, "{:indent$}--> ", "", indent = indent)
                })?;
                writeln!(self.out, "{file_name}:{line}:{col}",
                    file_name = start_pos_info.file.name(),
                    line = start_pos_info.line,
                    col = start_pos_info.col,
                )?;
                self.with_color(Color::Cyan, true, |out| {
                    writeln!(out, "{:indent$} |", "", indent = indent)
                })?;

                // This position is used to find current line source code.
                let mut pos = diag.span().start();
                for line in start_pos_info.line..=end_pos_info.line {
                    self.with_color(Color::Cyan, true, |out| {
                        write!(out, "{number:>indent$} | ",
                            number = line,
                            indent = indent)
                    })?;
                    // let line_source = source_map.lookup_line_source(pos);
                    // write!(self.out, "{}", line_source)?;

                    self.with_color(Color::Cyan, true, |out| {
                        writeln!(out, "{:indent$} |", "", indent = indent)
                    })?;

                    self.with_color(level_color, true, |out| {
                        write!(out, "{number:>indent$} | ",
                            number = line,
                            indent = indent)
                    })?;

                    let mut marks = "".to_string();
                    match line {
                        line if line == start_pos_info.line => {},
                        line if line == end_pos_info.line => {},

                        _ => {
                            let size =
                                source_map.lookup_line_bounds(pos).end().to_u32() -
                                source_map.lookup_line_bounds(pos).start().to_u32() + 1;
                            marks = level_mark_char.to_string().repeat(size as usize);
                        },
                    };

                    pos = source_map.lookup_line_bounds(pos).end() + 1u32;
                }

                self.with_color(Color::Cyan, true, |out| {
                    writeln!(out, "{:indent$} |", "", indent = indent)
                })?;
            }
        }
        Ok(())
    }

    // Some high-level helper functions for coloring the output. Maybe we can
    // create a new `tty` module for console pretty printing. But these wrappers
    // are enough for now.

    fn with_color<F>(&mut self, color: Color, bold: bool, mut f: F) -> io::Result<()>
        where F: FnMut(&mut StandardStream) -> io::Result<()>
    {
        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(color));
        color_spec.set_bold(bold);
        self.out.set_color(&color_spec)?;
        f(&mut self.out)?;
        self.out.reset()?;
        Ok(())
    }
}

impl Emitter for TtyEmitter {
    fn emit_diagnostic(&mut self, diag: &Diagnostic) {
        self.try_emit_diagnostic(diag).expect("error: failed to emit error.");
    }

    fn source_map(&self) -> Option<Rc<SourceMap>> {
        self.source_map.clone()
    }
}

pub struct SilentEmitter {

}
