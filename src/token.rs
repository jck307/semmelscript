#![allow(unused)]

use crate::*;
use std::fmt;

macro_rules! define_enums {
    ($($name:ident { $($field:ident $(= $str:expr)?,)* })*) => {
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

                pub const ARRAY: &[(Self, &'static str)] = &[$(
                    $(
                        (Self::$field, $str),
                    )?
                )*];
            }

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}", match self {
                        $(
                            $(
                                Self::$field => $str,
                            )?
                        )*
                        other => {
                            return fmt::Debug::fmt(other, f)
                        }
                    })
                }
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

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::String(value) => fmt::Display::fmt(value, f),
            Token::Integer(value) => fmt::Display::fmt(value, f),
            Token::Float(value) => fmt::Display::fmt(value, f),
            Token::Boolean(value) => fmt::Display::fmt(value, f),
            Token::Identifier(value) => fmt::Display::fmt(value, f),
            Token::Keyword(value) => fmt::Display::fmt(value, f),
            Token::Primitive(value) => fmt::Display::fmt(value, f),
            Token::Separator(value) => fmt::Display::fmt(value, f),
            Token::Operator(value) => fmt::Display::fmt(value, f),
        }
    }
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

#[derive(PartialEq, Debug)]
pub enum OperatorType {
    Equality,
    Boolean,
    Comparison,
    Arithmetic,
    Assignment,
    Range,
}

impl Operator {
    pub fn ty(&self) -> OperatorType {
        use Operator::*;
        use OperatorType::*;
        match self {
            Add |
            Sub |
            Mul |
            Div |
            Pow |
            Mod
                => Arithmetic,
            Equal |
            Inequal
                => Equality,
            Less |
            LessEqual |
            Greater |
            GreaterEqual
                => Comparison,
            And |
            Or |
            Not
                => Boolean,
            RangeExcl |
            RangeIncl
                => Range,
            Assign |
            AddAssign |
            SubAssign |
            MulAssign |
            DivAssign |
            PowAssign |
            ModAssign
                => Assignment,
        }
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
