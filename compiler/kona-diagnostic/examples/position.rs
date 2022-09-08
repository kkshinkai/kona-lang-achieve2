// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.
//
// $ cargo run --example position

use std::rc::Rc;

use kona_diagnostic::source::{SourceMap, Pos};

fn main() {
    let source_map = Rc::new(SourceMap::new());
    source_map.load_test_file(None, "abc\ndef\ngasd".to_string());
    source_map.load_test_file(None, "def".to_string());
    source_map.load_test_file(None, "def".to_string());

    println!("{:?}", source_map.lookup_line_at_pos(Pos::from_u32(12)));
}
