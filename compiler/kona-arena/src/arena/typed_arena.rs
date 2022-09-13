// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::{cell::RefCell, cmp, mem, iter};

// NOTE: `Vec<T>` may move to other memory during resize. We need to avoid this
// situation for our `Vec`-based arena.

pub struct TypedArena<T> {
    chunks: RefCell<ChunkList<T>>,
}

struct ChunkList<T> {
    current: Vec<T>,
    rest: Vec<Vec<T>>,
}

const INITIAL_SIZE: usize = 1024;
const MIN_CAPACITY: usize = 1;

impl<T> TypedArena<T> {
    pub fn new() -> TypedArena<T> {
        // TBD: How do we handle ZST here?
        let size = cmp::max(1, mem::size_of::<T>());
        TypedArena::with_capacity(INITIAL_SIZE / size)
    }

    /*pub*/ fn with_capacity(n: usize) -> TypedArena<T> {
        let n = cmp::max(MIN_CAPACITY, n);
        TypedArena {
            chunks: RefCell::new(ChunkList {
                current: Vec::with_capacity(n),
                rest: Vec::new(),
            }),
        }
    }

    #[inline]
    pub fn alloc(&self, value: T) -> &mut T {
        self.alloc_fast_path(value)
            .unwrap_or_else(|value| self.alloc_slow_path(value))
    }

    #[inline]
    fn alloc_fast_path(&self, value: T) -> Result<&mut T, T> {
        let mut chunks = self.chunks.borrow_mut();
        let len = chunks.current.len();
        if len < chunks.current.capacity() {
            // The fast path, we have enough space in the current chunk. Here
            // `push` should not cause a resize, otherwise the references we
            // emitted would become dangling.

            // FIXME: What happpened here?
            // debug_assert!(len < chunks.current.len());
            chunks.current.push(value);
            Ok(unsafe { &mut *chunks.current.as_mut_ptr().add(len) })
        } else {
            // Fast path failed, try to allocate a new chunk.
            Err(value)
        }
    }

    fn alloc_slow_path(&self, value: T) -> &mut T {
        &mut self.alloc_slice(iter::once(value))[0]
    }

    pub fn alloc_slice<I>(&self, iterable: I) -> &mut [T]
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iterable.into_iter();

        let mut chunks = self.chunks.borrow_mut();

        let iter_min_len = iter.size_hint().0;
        let mut next_item_index;
        if iter_min_len > chunks.current.capacity() - chunks.current.len() {
            chunks.reserve(iter_min_len);
            chunks.current.extend(iter);
            next_item_index = 0;
        } else {
            next_item_index = chunks.current.len();
            let mut i = 0;
            while let Some(elem) = iter.next() {
                if chunks.current.len() == chunks.current.capacity() {
                    let chunks = &mut *chunks;
                    chunks.reserve(i + 1);
                    let previous_chunk = chunks.rest.last_mut().unwrap();
                    let previous_chunk_len = previous_chunk.len();
                    chunks
                        .current
                        .extend(previous_chunk.drain(previous_chunk_len - i..));
                    chunks.current.push(elem);
                    chunks.current.extend(iter);
                    next_item_index = 0;
                    break;
                } else {
                    chunks.current.push(elem);
                }
                i += 1;
            }
        }
        let new_slice_ref = &mut chunks.current[next_item_index..];

        unsafe { mem::transmute::<&mut [T], &mut [T]>(new_slice_ref) }
    }
}

impl<T> ChunkList<T> {
    // Request a change in capacity.
    fn reserve(&mut self, additional: usize) {
        let double_cap = self.current
            .capacity()
            .checked_mul(2)
            .expect("capacity overflow");
        let required_cap = additional
            .checked_next_power_of_two()
            .expect("capacity overflow");
        let new_capacity = cmp::max(double_cap, required_cap);
        let chunk = mem::replace(&mut self.current, Vec::with_capacity(new_capacity));
        self.rest.push(chunk);
    }
}
