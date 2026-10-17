#![allow(unused)]

use {
    crate::{
        *,
        token::Operator,
    },
    std::mem::ManuallyDrop,
};

#[derive(Debug)]
pub enum Node {
    Block(Block),
    ParenArgs(Box<Node>, Vec<Node>),
    BinaryOp(Box<BinaryOp>),

    // Statements
    DefineVariable(Identifier, Box<Node>),
    DefineFunction(Identifier, Vec<Identifier>, Box<Node>),
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    For(Identifier, Box<Node>, Box<Node>),
    While(Box<Node>, Box<Node>),
    Label(Box<str>),
    Goto(Box<str>),

    // Literals
    Identifier(Identifier),
    String(Box<str>),
    Integer(Integer),
    Float(Float),
    Boolean(bool),
    List(Vec<Node>),
}

#[derive(Debug)]
pub struct BinaryOp {
    pub op: Operator,
    pub a: Node,
    pub b: Node,
}

#[derive(Debug)]
pub struct Block {
    pub nodes: Vec<Node>,
    pub return_last: bool,
}

#[derive(Debug, Clone)]
pub enum Identifier {
    String(Box<str>),
    Id(usize),
}

impl Identifier {
    pub fn as_string(&self) -> &Box<str> {
        let Self::String(value) = self else { panic!() };
        value
    }

    pub fn as_id(&self) -> &usize {
        let Self::Id(value) = self else { panic!() };
        value
    }
}

impl From<String> for Identifier {
    fn from(string: String) -> Self {
        Self::String(string.into())
    }
}

// #[derive(Debug, Clone)]
// pub enum NodeType {
//     Block,
//     ParenArgs,
//     BinaryOp,
//     DefineVariable,
//     DefineFunction,
//     If,
//     For,
//     While,
//     Label,
//     Goto,
//     Identifier,
//     String,
//     Integer,
//     Float,
//     Boolean,
//     List,
// }

impl Node {
    pub fn name(&self) -> &'static str {
        match self {
            Node::Block(..) => "Block",
            Node::ParenArgs(..) => "ParenArgs",
            Node::BinaryOp(..) => "BinaryOp",
            Node::DefineVariable(..) => "DefineVariable",
            Node::DefineFunction(..) => "DefineFunction",
            Node::If(..) => "If",
            Node::For(..) => "For",
            Node::While(..) => "While",
            Node::Label(..) => "Label",
            Node::Goto(..) => "Goto",
            Node::Identifier(..) => "Identifier",
            Node::String(..) => "String",
            Node::Integer(..) => "Integer",
            Node::Float(..) => "Float",
            Node::Boolean(..) => "Boolean",
            Node::List(..) => "List",
        }
    }

    // pub fn nodetype(&self) -> NodeType {
    //     use NodeType::*;
    //     match self {
    //         Node::Block(..) => Block,
    //         Node::ParenArgs(..) => ParenArgs,
    //         Node::BinaryOp(..) => BinaryOp,
    //         Node::DefineVariable(..) => DefineVariable,
    //         Node::DefineFunction(..) => DefineFunction,
    //         Node::If(..) => If,
    //         Node::For(..) => For,
    //         Node::While(..) => While,
    //         Node::Label(..) => Label,
    //         Node::Goto(..) => Goto,
    //         Node::Identifier(..) => Identifier,
    //         Node::String(..) => String,
    //         Node::Integer(..) => Integer,
    //         Node::Float(..) => Float,
    //         Node::Boolean(..) => Boolean,
    //         Node::List(..) => List,
    //     }
    // }
}
