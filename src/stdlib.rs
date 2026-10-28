#![allow(unused)]

mod funcs;
pub use funcs::*;

use {
    std::collections::HashMap,
    crate::{
        instruction::{Instruction, Function},
        types::{Type, FunctionSignature},
    },
};

pub type Builtins = HashMap<Box<str>, (Function, FunctionSignature)>;

macro_rules! add {
    ($hashmap:expr, [$($name:ident($($arg:ident),*) $(-> $return:ident)?,)*]) => {{
        $(
            let ptr: fn(&mut crate::runtime::Runtime) = $name;
            let func = vec![Instruction::CallInternal(ptr)];
            let mut sig = FunctionSignature::default();
            $(
                sig.0.push(Type::$arg);
            )*
            $(
                sig.1.push(Type::$return);
            )?
            $hashmap.insert(stringify!($name).into(), (func, sig));
        )*
    }}
}

pub fn get_builtins() -> Builtins {
    let mut builtins = HashMap::new();
    add!(builtins, [
        println(String),
        tostring(Any) -> String,
    ]);
    builtins
}
