// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::{rc::Rc, sync::Mutex};

use crate::source::{SourceMap, Span};

use crate::diagnostic::{TtyEmitter, Emitter, DiagnosticBuilder, Diagnostic, Level};

pub struct DiagnosticEngine {
    pub(crate) inner: Mutex<DiagnosticEngineInner>,
}

pub struct DiagnosticEngineInner {
    // NOTE: I was going to use `Vec<Box<dyn Emitter>>` here, but then I
    //       realized that the composability of emitters is more important than
    //       looping through multiple separate emitters. We should implement
    //       something like `emitter1.compose(emitter2)` instead of just
    //       `vec![emitter1, emitter2]`.
    emitter: Box<dyn Emitter>,
}

impl DiagnosticEngine {
    pub fn with_emitter(emitter: Box<dyn Emitter>) -> DiagnosticEngine {
        DiagnosticEngine {
            inner: Mutex::new(DiagnosticEngineInner {
                emitter,
            })
        }
    }

    pub fn with_tty_emitter(source_map: Rc<SourceMap>) -> DiagnosticEngine {
        DiagnosticEngine::with_emitter(
            Box::new(TtyEmitter::new(source_map)),
        )
    }

    pub fn create_diagnostic(&self, level: Level, msg: impl Into<String>) -> DiagnosticBuilder<()> {
        DiagnosticBuilder::new(
            self,
            Box::new(Diagnostic::new(level, Span::dummy(), msg.into())),
        )
    }

    pub fn create_err(&self, msg: impl Into<String>) -> DiagnosticBuilder<()> {
        DiagnosticBuilder::new(
            self,
            Box::new(Diagnostic::new(Level::Error, Span::dummy(), msg.into())),
        )
    }

    pub fn create_warn(&self, msg: impl Into<String>) -> DiagnosticBuilder<()> {
        DiagnosticBuilder::new(
            self,
            Box::new(Diagnostic::new(Level::Warn, Span::dummy(), msg.into())),
        )
    }

    pub fn create_note(&self, msg: impl Into<String>) -> DiagnosticBuilder<()> {
        DiagnosticBuilder::new(
            self,
            Box::new(Diagnostic::new(Level::Note, Span::dummy(), msg.into())),
        )
    }

    pub fn emit_diagnostic(&self, diagnostic: &Diagnostic) {
        let mut inner = self.inner.lock().unwrap();
        inner.emitter.emit_diagnostic(diagnostic)
    }
}
