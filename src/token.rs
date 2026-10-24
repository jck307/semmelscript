#![allow(unused)]

use crate::*;

macro_rules! define_enums {
    (
        $($name:ident { $($field:ident $(= $str:expr)?,)* })*
    ) => {
        $(
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
                            $(
                                $str => Some(Self::$field),
                            )?
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
                    $(
                        (Self::$field, $str),
                    )?
                )*];
            }
        )*

        pub mod consts {
            #![allow(non_upper_case_globals)]
            $(
                $(
                    pub const $field: super::Token
                        = super::Token::$name(super::$name::$field);
                )*
            )*
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
    Primitive(Primitive),
    Separator(Separator),
    Operator(Operator),
    // Underscore,
}

define_enums! {
    Keyword {
        Let = "let",
        If = "if",
        Else = "else",
        Elif = "elif",
        Func = "func",
        True = "true",
        False = "false",
        For = "for",
        While = "while",
        In = "in",
        Goto = "goto",
        KwAnd = "and",
        KwOr = "or",
        KwNot = "not",
    }

    Primitive {
        Int = "int",
        Uint = "uint",
        Float = "float",
        Bool = "bool",
        String = "str",
    }

    Separator {
        ParenOpen = "(",
        ParenClose = ")",
        BracketOpen = "[",
        BracketClose = "]",
        BraceOpen = "{",
        BraceClose = "}",
        Comma = ",",
        Colon = ":",
        Semicolon = ";",
    }

    Operator {
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
        And,
        Or,
        Not,

        // ranges
        RangeExcl = "..",
        RangeIncl = "..=",

        // assign
        Assign = "=",
        AddAssign = "+=",
        SubAssign = "-=",
        MulAssign = "*=",
        DivAssign = "/=",
        PowAssign = "^=",
        ModAssign = "%=",
    }
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

pub static UNARY_OPERATORS: &[Operator] = &[
    Not,
];

pub static BOOLEAN_OPERATORS: &[Operator] = &[
    Equal,
    Inequal,
    And,
    Or,
];

pub static COMPARISON_OPERATORS: &[Operator] = &[
    Equal,
    Inequal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
];

pub static ARITHMETIC_OPERATORS: &[Operator] = &[
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Mod,
];
