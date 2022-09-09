// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

//! A diagnostic system for source text management and error reporting.
//!
//! This diagnostic system consists of two parts:
//!
//! - A **source text management system** (in the [`source`] module). It provides a
//!     mechanism to analyze and cache position information of source text.
//!     After loading the source files into [`SourceMap`], you can query the
//!     information like the file name, line number, column number, etc. of a
//!     specified position [`Pos`] that is represented by [`u32`];
//! - A **diagnostic engine** (inthe  [`diagnostic`] module) generates rich
//!     diagnostic messages. A diagnostic [`Diagnostic`] consists of some
//!     positions and message strings. Diagnostic engines can emit it to
//!     different formats, print to console, write to file, or export it as
//!     JSON.
//!
//! Most instances of the diagnostic engine require a source map to output more
//! precise error messages.
//!
//! ## Examples
//!
//! Here is a simple example of using the diagnostic engine. We first load a
//! source file `example.sml` into a source map, and then create a diagnostic
//! engine with TTY emitter. Finally, we build a diagnostic at `22..24` and
//! emit it.
//!
//! ```rust,ignore
//! let source_map = Rc::new(SourceMap::new());
//! source_map
//!     .load_local_file(PathBuf::from("example.sml"))
//!     .expect("failed to read the example file");
//!
//! let engine = DiagnosticEngine::with_tty_emitter(source_map);
//! engine
//!     .create_err("unexpected token keyword `in` in case-of expression")
//!     .set_primary_label(22..24u32, "expect keyword `of`")
//!     .emit();
//! ```
//!
//! The output should be:
//!
//! ```text
//! error: unexpected token keyword `in` in case-of expression
//!  --> example.sml:2:6
//!   |
//! 5 |    case n in 0 => 0
//!   |           ^^ expect keyword `of`
//! ```
//!
//! Note that cast `u32` into [`Pos`] without checking is dangerous. We create a
//! span from `22..24u32` in this small example, but it is not a good practice.
//!
//! [`SourceMap`]: source::SourceMap
//! [`Pos`]: source::Pos
//! [`Diagnostic`]: diagnostic::Diagnostic
pub mod source;
pub mod diagnostic;
