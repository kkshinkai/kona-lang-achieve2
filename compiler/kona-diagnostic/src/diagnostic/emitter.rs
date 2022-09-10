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
            if let Ok(lines) = source_map.lookup_lines_at_span(diag.span()) {
                if lines.is_empty() {
                    return Ok(());
                }

                let indent = lines.last().unwrap().line_number().to_string().len();
                let start_pos_info = source_map
                    .lookup_pos_info(diag.span().start())
                    .unwrap();
                let end_pos_info = source_map
                    .lookup_pos_info(diag.span().end())
                    .unwrap();

                // Prints "  --- src/main.sml:1:1"
                self.with_color(Color::Cyan, true, |out| {
                    // NOTE: VS Code problem matcher (`"^[\\s->=]*(.*?):(\\d*):(\\d*)\\s*$"`)
                    // may parse `"--> <file>:<n>:<m>"` as a Rustc error. We
                    // need to avoid this.~
                    write!(out, "{:indent$} .- ", "", indent = indent)
                })?;
                writeln!(self.out, "{file_name}:{line}:{col}",
                    file_name = start_pos_info.name(),
                    line = start_pos_info.line(),
                    col = start_pos_info.col(),
                )?;
                self.with_color(Color::Cyan, true, |out| {
                    writeln!(out, "{:indent$} |", "", indent = indent)
                })?;

                for (idx, line) in lines.iter().enumerate() {
                    self.with_color(Color::Cyan, true, |out| {
                        write!(out, "{number:>indent$} | ",
                            number = line.line_number(),
                            indent = indent)
                    })?;

                    let line_source = line.source();
                    write!(self.out, "{}", line_source)?;

                    self.with_color(Color::Cyan, true, |out| {
                        write!(out, "{:indent$} | ", "", indent = indent)
                    })?;

                    let span = line.span();

                    let mark_start_col =
                        if idx == 0 {
                           start_pos_info.col_display()
                        } else {
                            0
                        };
                    let mark_end_col =
                        if idx == lines.len() - 1 {
                            end_pos_info.col_display()
                        } else {
                            source_map
                                .lookup_pos_info(span.end() - 1u32)
                                .unwrap()
                                .col_display()
                        };

                    let marks_line = " ".repeat(mark_start_col)
                        + &level_mark_char.to_string().repeat(mark_end_col - mark_start_col)
                        + " " + &diag.labels.primary_label.message;
                    self.with_color(level_color, true, |out| {
                        writeln!(out, "{}", marks_line)
                    })?;
                }

                self.with_color(Color::Cyan, true, |out| {
                    writeln!(out, "{:indent$} '-", "", indent = indent)
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
