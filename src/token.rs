#![allow(unused)]

use crate::*;

macro_rules! string_enum {
    ($name:ident, $($field:ident = $str:expr,)*) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum $name {
            $(
                $field,
            )*
        }

        impl $name {
            pub fn try_from_str(string: &str) -> Option<Self> {
                match string {
                    $(
                        $str => Some(Self::$field),
                    )*
                    _ => None
                }
            }

            // pub fn as_str(&self) -> &'static str {
            //     match self {
            //         $(
            //             Self::$field => $str,
            //         )*
            //     }
            // }

            pub const ARRAY: &[(Self, &'static str)] = &[$(
                (Self::$field, $str),
            )*];
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    String(String),
    Integer(Integer),
    Float(Float),
    Boolean(bool),
    Identifier(String),
    Keyword(Keyword),
    Operator(Operator),
}

string_enum! { Keyword,
    Let = "let",
    If = "if",
    Else = "else",
    Elif = "elif",
    Func = "fn",
    True = "true",
    False = "false",
    For = "for",
    While = "while",
    In = "in",
}

string_enum! { Operator,
    // misc
    ParenOpen = "(",
    ParenClose = ")",
    BracketOpen = "[",
    BracketClose = "]",
    BraceOpen = "{",
    BraceClose = "}",
    Dot = ".",
    Comma = ",",
    Colon = ":",
    Semicolon = ";",
    Dollar = "$",
    Exclamation = "!",
    Question = "?",
    Hashtag = "#",

    // math
    Add = "+",
    Sub = "-",
    Mul = "*",
    Div = "/",
    Pow = "^",
    Mod = "%",

    // compare
    Equal = "==",
    Inequal = "!=",
    Less = "<",
    LessEqual = "<=",
    Greater = ">",
    GreaterEqual = ">=",

    // logic
    And = "&&",
    Or = "||",

    // ranges
    RangeExcl = "..",
    RangeIncl = "..=",

    // assign
    Assign = "=",
    AddAssign = "+=",
    SubAssign = "-=",
    MulAssign = "*=",
    DivAssign = "/=",
    PowAssign = "^⁼",
    ModAssign = "%=",
}

use Operator::*;

pub static OPERATOR_ORDER: &[&[Operator]] = &[
    &[RangeExcl, RangeIncl],
    &[Pow],
    &[Mul, Div, Mod],
    &[Add, Sub],
    &[Equal, Inequal, Less, LessEqual, Greater, GreaterEqual],
    &[And],
    &[Or],
    &[Assign, AddAssign, SubAssign, MulAssign, DivAssign, PowAssign, ModAssign],
];

pub static BINARY_OPERATORS: &[Operator] = &[
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
    Equal,
    Inequal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    RangeExcl,
    RangeIncl,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    PowAssign,
    ModAssign,
];
