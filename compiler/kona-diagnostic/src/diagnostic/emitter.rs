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
    fn source_map(&self) -> Option<&Rc<SourceMap>>;
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
       let mut color_spec = ColorSpec::new();

       color_spec.set_fg(Some(match diag.level {
           Level::Error => Color::Red,
           Level::Warn => Color::Yellow,
           Level::Note => Color::Blue,
       }));
       color_spec.set_bold(true);

       self.out.set_color(&color_spec)?;
       write!(self.out, "{}", match diag.level {
           Level::Error => "error",
           Level::Warn => "warning",
           Level::Note => "note",
       })?;

       color_spec.set_fg(None);
       color_spec.set_bold(false);
       self.out.set_color(&color_spec)?;
       writeln!(self.out, ": {}.", diag.message)?;

        // Prints the source location if available.
        if let Some(source_map) = self.source_map() {
            let pos_info = source_map.lookup_pos_info(diag.span().start());
            let line_number = pos_info.line.to_string();
            let indent = line_number.len() + 2;

            // Prints "  --> src/main.rs:1:1"
            write!(self.out, "{:indent$}--> {file_name}:{line}:{col}",
                "", indent = indent,
                file_name = pos_info.file.name(),
                line = pos_info.line,
                col = pos_info.col,
            )?;
        }
        Ok(())
    }
}

impl Emitter for TtyEmitter {
    fn emit_diagnostic(&mut self, diag: &Diagnostic) {
        self.try_emit_diagnostic(diag).expect("error: failed to emit error.");
    }

    fn source_map(&self) -> Option<&Rc<SourceMap>> {
        self.source_map.as_ref()
    }
}

pub struct SilentEmitter {

}
