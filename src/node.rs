use {
    crate::{
        *,
        token::Operator,
    },
    types::Type,
};

#[allow(unused)]
#[derive(Debug)]
pub enum Node {
    Block(Block, NodeMeta),
    ParenArgs(Box<Node>, Vec<Node>, NodeMeta),
    BinaryOp(Box<BinaryOp>, NodeMeta),
    UnaryOp(Box<UnaryOp>, NodeMeta),

    // Statements
    DefineVariable(Identifier, Box<Node>, NodeMeta),
    DefineFunction(Identifier, Vec<Identifier>, Type, Box<Node>, NodeMeta),
    If(Vec<(Box<Node>, Box<Node>)>, Option<Box<Node>>, NodeMeta),
    For(Identifier, Box<Node>, Box<Node>, NodeMeta),
    While(Box<Node>, Box<Node>, NodeMeta),
    Label(Box<str>, NodeMeta),
    Goto(Box<str>, NodeMeta),

    // Literals
    Identifier(Identifier, NodeMeta),
    String(Box<str>, NodeMeta),
    Integer(Integer, NodeMeta),
    Float(Float, NodeMeta),
    Boolean(bool, NodeMeta),
    List(Vec<Node>, NodeMeta),
}

#[derive(Debug, Clone)]
pub struct NodeMeta {
    #[allow(dead_code)]
    pub location: Option<(u16, u16)>,
    pub ty: Type,
}

impl NodeMeta {
    pub fn empty() -> Self {
        Self {
            location: None,
            ty: Type::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct BinaryOp {
    pub op: Operator,
    pub a: Node,
    pub b: Node,
}

#[derive(Debug)]
pub struct UnaryOp {
    pub op: Operator,
    pub value: Node,
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
    pub fn meta(&self) -> &NodeMeta {
        match self {
            Node::Block(.., meta) => meta,
            Node::ParenArgs(.., meta) => meta,
            Node::BinaryOp(.., meta) => meta,
            Node::UnaryOp(.., meta) => meta,
            Node::DefineVariable(.., meta) => meta,
            Node::DefineFunction(.., meta) => meta,
            Node::If(.., meta) => meta,
            Node::For(.., meta) => meta,
            Node::While(.., meta) => meta,
            Node::Label(.., meta) => meta,
            Node::Goto(.., meta) => meta,
            Node::Identifier(.., meta) => meta,
            Node::String(.., meta) => meta,
            Node::Integer(.., meta) => meta,
            Node::Float(.., meta) => meta,
            Node::Boolean(.., meta) => meta,
            Node::List(.., meta) => meta,
        }
    }

    // pub fn name(&self) -> &'static str {
    //     match self {
    //         Node::Block(..) => "Block",
    //         Node::ParenArgs(..) => "ParenArgs",
    //         Node::BinaryOp(..) => "BinaryOp",
    //         Node::DefineVariable(..) => "DefineVariable",
    //         Node::DefineFunction(..) => "DefineFunction",
    //         Node::If(..) => "If",
    //         Node::For(..) => "For",
    //         Node::While(..) => "While",
    //         Node::Label(..) => "Label",
    //         Node::Goto(..) => "Goto",
    //         Node::Identifier(..) => "Identifier",
    //         Node::String(..) => "String",
    //         Node::Integer(..) => "Integer",
    //         Node::Float(..) => "Float",
    //         Node::Boolean(..) => "Boolean",
    //         Node::List(..) => "List",
    //     }
    // }

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
