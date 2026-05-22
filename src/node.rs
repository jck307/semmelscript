#![allow(unused)]

use crate::*;
use crate::token::Operator;

#[derive(Debug, Clone)]
pub enum Node {
    Statement(Statement),
    Block(Block),
    ParenArgs(Box<Node>, Vec<Node>),

    // Operators
    BinaryOp(Box<BinaryOp>),

    Identifier(Box<str>),
    String(Box<str>),
    Integer(Integer),
    Float(Float),
    Boolean(bool),
    List(Vec<Node>),
}

#[derive(Debug, Clone)]
pub struct BinaryOp {
    pub op: Operator,
    pub a: Node,
    pub b: Node,
}

#[derive(Debug, Clone)]
pub enum Statement {
    // TODO replace String with Box<str>
    DefineVariable(String, Box<Node>),
    DefineFunction(String, Vec<Box<str>>, Block),
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    For(Box<str>, Box<Node>, Box<Node>),
    While(Box<Node>, Box<Node>),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub nodes: Vec<Node>,
}
