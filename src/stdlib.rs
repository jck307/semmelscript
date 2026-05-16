use crate::*;
use crate::runtime::*;

mod funcs;
use funcs::*;

#[macro_export]
macro_rules! get {
    ($runtime:expr, $scope:expr, $name:ident, $type:ident) => {
        expect_type!(
            $scope.get($runtime, stringify!($name))
                .unwrap_or_else(|_| {
                    panic!(concat!("invalid arg: ", stringify!($name)));
                }),
            $type
        )
    }
}

use crate::get;

macro_rules! add {
    ($scope:expr,
        $($name:ident($($arg:ident$(,)?)*);)*) => {

        $(
            $scope.define(stringify!($name),
                Object::Function {
                    func: Box::new(Function::Pointer($name)),
                    args: vec![$( stringify!($arg).into(), )*],
                }
            );
        )*
    }
}

pub fn init(scope: &mut Scope) {
    // this is such a sexy macro
    add!(scope,
        println(text);
        print(text);
        call(cmd);
        source(path);
        tostring(value);
    );
}
