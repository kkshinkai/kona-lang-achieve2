// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::marker::PhantomData;

use crate::source::Span;

use crate::{diagnostic::Diagnostic};

use super::DiagnosticEngine;

pub struct DiagnosticBuilder<'a, G: EmissionGuarantee> {
    state: DiagnosticBuilderState<'a>,
    diagnostic: Box<Diagnostic>,
    _marker: PhantomData<G>,
}

impl<'a, G: EmissionGuarantee> DiagnosticBuilder<'a, G> {
    pub fn new(
        engine: &'a DiagnosticEngine,
        diagnostic: Box<Diagnostic>,
    ) -> DiagnosticBuilder<'a, G> {
        DiagnosticBuilder {
            state: DiagnosticBuilderState::Emittable(engine),
            diagnostic,
            _marker: PhantomData,
        }
    }

    pub fn set_span(mut self, span: impl Into<Span>) -> DiagnosticBuilder<'a, G> {
        self.diagnostic.span = span.into();
        self
    }

    pub fn emit(mut self) -> G {
        G::emit_diagnostic_with_guarantee(&mut self)
    }

    pub fn cancel(mut self) {
        self.state = DiagnosticBuilderState::EmittedOrCancelled;
        drop(self);
    }
}

/// A destructor bomb, a `DiagnosticBuilder` must be either emitted or cancelled
/// or we report a bug.
impl<'a, G: EmissionGuarantee> Drop for DiagnosticBuilder<'a, G> {
    fn drop(&mut self) {
        match self.state {
            DiagnosticBuilderState::Emittable(_) => {
                panic!("`DiagnosticBuilder` was not emitted or cancelled");
            }
            DiagnosticBuilderState::EmittedOrCancelled => {}
        }
    }
}

pub enum DiagnosticBuilderState<'a> {
    Emittable(&'a DiagnosticEngine),
    EmittedOrCancelled,
}

pub trait EmissionGuarantee: Sized {
    fn emit_diagnostic_with_guarantee(
        db: &mut DiagnosticBuilder<Self>
    ) -> Self;
}

pub struct EmissionGuaranted;

impl EmissionGuarantee for EmissionGuaranted {
    fn emit_diagnostic_with_guarantee(
        db: &mut DiagnosticBuilder<Self>,
    ) -> Self {
        match db.state {
            DiagnosticBuilderState::Emittable(engine) => {
                engine.emit_diagnostic(db.diagnostic.as_ref());
                db.state = DiagnosticBuilderState::EmittedOrCancelled;
                EmissionGuaranted
            }

            DiagnosticBuilderState::EmittedOrCancelled => {
                panic!("`DiagnosticBuilder` was not emitted or cancelled");
            }
        }
    }
}

impl EmissionGuarantee for () {
    fn emit_diagnostic_with_guarantee(db: &mut DiagnosticBuilder<Self>) -> Self {
        match db.state {
            DiagnosticBuilderState::Emittable(handler) => {
                db.state = DiagnosticBuilderState::EmittedOrCancelled;

                handler.emit_diagnostic(&db.diagnostic);
            }
            // `.emit()` was previously called, disallowed from repeating it.
            DiagnosticBuilderState::EmittedOrCancelled => {}
        }
    }
}

