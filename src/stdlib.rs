#![allow(unused)]

mod funcs;
pub use funcs::*;

use {
    std::collections::HashMap,
    crate::{
        instruction::Instruction,
        types::Type,
    },
};

pub type Builtins = HashMap<Box<str>, Vec<Instruction>>;

macro_rules! add {
    ($hashmap:expr, [$($name:ident($($arg:ident),*),)*]) => {{
        $(
            let ptr: fn(&mut crate::runtime::Runtime) = $name;
            let func = vec![Instruction::CallInternal(ptr)];
            $hashmap.insert(stringify!($name).into(), func);
        )*
    }}
}

pub fn get_builtins() -> Builtins {
    let mut builtins = HashMap::new();
    add!(builtins, [
        println(),
        tostring(),
    ]);
    builtins
}
