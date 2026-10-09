#![allow(unused)]

mod funcs;

pub use funcs::*;
use std::collections::HashMap;

pub type Builtin = fn(&mut crate::Runtime);
pub type Builtins = HashMap<Box<str>, Builtin>;

macro_rules! add {
    ($hashmap:expr, [$($name:ident($($arg:ident),*),)*]) => {
        $(
            let ptr: Builtin = $name;
            $hashmap.insert(stringify!($name).into(), ptr);
        )*
    }
}

pub fn get_builtins() -> Builtins {
    let mut builtins = HashMap::new();
    add!(builtins, [
        println(),
        tostring(),
    ]);
    builtins
}
