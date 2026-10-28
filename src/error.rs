use {
    std::{
        error::Error,
        fmt::{
            Debug,
            Display,
            Formatter,
        },
    },
    crate::token::{self, Token},
};

pub trait ErrDisplay: Display + Debug {
    fn errfmt(&self) -> String;
}

type Item = Box<dyn ErrDisplay>;

#[derive(Debug)]
pub enum SyntaxError {
    EOF,
    Expected(Item, Item),
    ExpectedN(Vec<Item>, Item),
    Unexpected(Item),
    InvalidOperator(Box<str>),
    UnexpectedCharacter(char),
    // InvalidIdentifier(Box<str>),
}

impl Error for SyntaxError {}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            SyntaxError::EOF
                => "unexpected end of file".to_string(),
            SyntaxError::Expected(a, b)
                => format!("expected {}, found {}", a.errfmt(), b.errfmt()),
            SyntaxError::ExpectedN(items, b) => {
                let mut items_str = String::new();
                for (i, item) in items.iter().enumerate() {
                    items_str.push_str(&item.errfmt());
                    if i == items.len()-2 {
                        items_str.push_str(" or ");
                    } else if i != items.len()-1 {
                        items_str.push_str(", ");
                    }
                }
                format!("expected {items_str}, found {}", b.errfmt())
            }
            SyntaxError::Unexpected(item)
                => format!("unexpected {}", item.errfmt()),
            SyntaxError::UnexpectedCharacter(ch)
                => format!("unexpected character `{ch}`"),
            SyntaxError::InvalidOperator(string)
                => format!("invalid operator `{string}`"),
            // SyntaxError::InvalidIdentifier(string)
            //     => format!("invalid identifier `{string}`")
        })
    }
}

impl ErrDisplay for char {
    fn errfmt(&self) -> String {
        format!("`{self}`")
    }
}

impl ErrDisplay for Token {
    fn errfmt(&self) -> String {
        format!("`{self}`")
    }
}

impl ErrDisplay for &str {
    fn errfmt(&self) -> String {
        format!("{self}")
    }
}

impl ErrDisplay for token::Separator {
    fn errfmt(&self) -> String {
        format!("`{self}`")
    }
}

impl ErrDisplay for token::Operator {
    fn errfmt(&self) -> String {
        format!("`{self}`")
    }
}
