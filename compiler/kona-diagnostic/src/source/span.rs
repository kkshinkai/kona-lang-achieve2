// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::{ops::Range, fmt};

use super::Pos;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    start: Pos,
    end: Pos,
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.start.to_u32(), self.end.to_u32())
    }
}

impl Span {
    pub fn new(start: Pos, end: Pos) -> Span {
        debug_assert!(start.to_u32() != 0 && end.to_u32() != 0,
            "position `Pos::from_u32(0)' is reserved for dummy span");

        Span { start, end }
    }

    pub fn dummy() -> Span {
        Span {
            start: Pos::from_u32(0),
            end: Pos::from_u32(0),
        }
    }

    pub fn is_dummy(&self) -> bool {
        (self.start().to_u32(), self.end().to_u32()) == (0, 0)
    }

    pub fn start(&self) -> Pos {
        self.start
    }

    pub fn end(&self) -> Pos {
        self.end
    }

    /// Merges two spans into one. The new span will include the gap between the
    /// two spans.
    ///
    /// This function usually used to calculate the span of an AST node from the
    /// span of the start and end token.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kona_diagnostic::source::Span;
    /// assert_eq!(Span::from(2..5u32).across(Span::from(8..10u32)), Span::from(2..10u32));
    /// assert_eq!(Span::from(3..9u32).across(Span::from(8..15u32)), Span::from(3..15u32));
    /// ```
    pub fn across(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn cross_over(&self, other: Span) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn contains(&self, other: Pos) -> bool {
        self.start <= other && other <= self.end
    }
}

impl From<Range<Pos>> for Span {
    fn from(rng: Range<Pos>) -> Self {
        Span::new(rng.start, rng.end)
    }
}

// NOTE: Implementing traits such as `From<Range<u32>` for `Span` may not be a
// good idea. Without salt like `Pos::from_u32` or `Pos::from_usize`, users may
// ignore the risk of converting `u32` or `usize` to `Pos`.
//
// I implemented these traits for convenient. It is nice to have `a..b` instead
// of `Span::new(Pos::from_usize(a), Pos::from_usize(b))` anyway.

impl From<Range<u32>> for Span {
    fn from(rng: Range<u32>) -> Self {
        Span::new(
            Pos::from_u32(rng.start),
            Pos::from_u32(rng.end),
        )
    }
}

impl From<Range<usize>> for Span {
    fn from(rng: Range<usize>) -> Self {
        Span::new(
            Pos::from_usize(rng.start),
            Pos::from_usize(rng.end),
        )
    }
}

impl From<(u32, u32)> for Span {
    fn from(rng: (u32, u32)) -> Self {
        Span::new(
            Pos::from_u32(rng.0),
            Pos::from_u32(rng.1),
        )
    }
}

impl From<(usize, usize)> for Span {
    fn from(rng: (usize, usize)) -> Self {
        Span::new(
            Pos::from_usize(rng.0),
            Pos::from_usize(rng.1),
        )
    }
}

impl Default for Span {
    fn default() -> Self {
        Span::dummy()
    }
}
