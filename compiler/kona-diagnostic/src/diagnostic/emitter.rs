// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::rc::Rc;

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

impl Emitter for TtyEmitter {
    fn emit_diagnostic(&mut self, diag: &Diagnostic) {
        // Prints colored "error", "warning", or "note".
        let mut color_spec = ColorSpec::new();

        color_spec.set_fg(Some(match diag.level {
            Level::Error => Color::Red,
            Level::Warn => Color::Yellow,
            Level::Note => Color::Blue,
        }));
        color_spec.set_bold(true);

        // Prints colored "error", "warning", or "note".
        self.out.set_color(&color_spec).expect("error: failed to emit error");
        write!(self.out, "{}", match diag.level {
            Level::Error => "error",
            Level::Warn => "warning",
            Level::Note => "note",
        }).unwrap();

        color_spec.set_fg(None);

        // Prints the message.
        self.out.set_color(&color_spec).expect("error: failed to emit error");
        writeln!(self.out, ": {}", diag.message).unwrap();

        // Resets the color and boldness.
        self.out.reset().expect("error: failed to emit error");

        // Prints the source location if available.
        if let Some(source_map) = self.source_map() {
            if diag.span().is_dummy() || diag.level == Level::Note {
                return;
            }
            let info = source_map.lookup_pos_info(diag.span().start());
            writeln!(self.out, "    File {file}:{line}:{col}",
                file = info.file.name(),
                line = info.line, col = info.col,
            ).expect("error: failed to emit error");
        }
    }

    fn source_map(&self) -> Option<&Rc<SourceMap>> {
        self.source_map.as_ref()
    }
}

pub struct SilentEmitter {

}
