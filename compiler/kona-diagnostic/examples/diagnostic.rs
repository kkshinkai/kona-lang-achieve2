// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.
//
// $ cargo run --example diagnostic

use std::{path::PathBuf, rc::Rc};

use kona_diagnostic::{source::{SourceMap, Span}, diagnostic::DiagnosticEngine};

fn main() {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/example.sml");
    let source_map = Rc::new(SourceMap::new());
    source_map
        .load_local_file(file.clone())
        .expect(&format!("failed to read {}", file.to_string_lossy()));

    let engine = DiagnosticEngine::with_tty_emitter(source_map);

    engine.create_err("this is an error message")
        .set_primary_label(5..10u32, "this is a primary label")
        .emit();
}
