// Copyright (c) Kk Shinkai. All Rights Reserved. See LICENSE.txt in the project
// root for license information.

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use bumpalo::Bump;

/// An internalized string.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol {
    pub(self) id: u32,
}

impl Symbol {
    /// Interns the given string and returns its symbol representation. If the
    /// string is already interned, returns the existing symbol.
    pub fn intern(string: &str) -> Symbol {
        GLOBAL_INTERNER.lock().unwrap().intern(string)
    }

    /// Returns the reference of underlying string.
    pub fn as_str(&self) -> &str {
        // SAFETY: The lifetime of the return value is the same as `&self`, but
        //         actually tied to the lifetime of the underlying interner.
        //         Interners are long-lived, this cast is safe.
        unsafe {
            std::mem::transmute::<&str, &str>(
                GLOBAL_INTERNER.lock().unwrap().get(*self),
            )
        }
    }

    /// Make a copy of the underlying string.
    ///
    /// TBD: This function is really useless, we never used it in our code. Even
    ///      if we do need a copy, we can use `symbol.as_str().to_string()`
    ///      instead.
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

use lazy_static::lazy_static;

lazy_static! {
    static ref GLOBAL_INTERNER: Mutex<Interner> =
        Mutex::new(Interner::default());
}

#[derive(Default)]
struct Interner {
    // FIXME: The storage may resize, fix this immediately!
    storage: Bump,
    symbols: Vec<&'static str>,
    names: HashMap<&'static str, Symbol>,
}

impl Interner {
    fn intern(&mut self, string: &str) -> Symbol {
        if let Some(&symbol) = self.names.get(string) {
            return symbol;
        }

        let symbol = Symbol { id: self.symbols.len() as u32 };

        // SAFETY: We put the string into the interner, and never remove item
        //         from it. It is safe to use `&'static str` here.
        let str: &'static str = unsafe {
            let str: &str = self.storage.alloc_str(string);
            std::mem::transmute::<&str, &'static str>(str)
        };

        self.names.insert(str, symbol);
        self.symbols.push(str);
        symbol
    }

    fn get(&self, symbol: Symbol) -> &str {
        &self.symbols[symbol.id as usize]
    }
}
