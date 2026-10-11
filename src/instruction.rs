use crate::{
    types::*,
    runtime::*,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Instruction {
    Debug,
    Exit,
    Call(Function),
    CallInternal(fn(&mut Runtime)),
    Goto(usize),
    GotoIf(usize),
    LoopUpdate(u64, u64, usize),
    SetStackIndex(usize),
    Pop,
    Dup(usize),
    Swap(usize),
    MoveTo(usize),
    RotateLeft(usize),
    RotateRight(usize),
    PushNumber8(Type, u8),
    PushNumber16(Type, u16),
    PushNumber32(Type, u32),
    PushNumber64(Type, u64),
    PushBool(bool),
    PushString(String),
    PushType(Type),
    Add(Type),
    Sub(Type),
    Mul(Type),
    Div(Type),
    Mod(Type),
    Equal(Type),
    Inequal(Type),
    Greater(Type),
    GreaterEqual(Type),
    Less(Type),
    LessEqual(Type),
    And,
    Or,
    Placeholder,
}

use Instruction::*;

impl Instruction {
    pub fn get_stack_diff(&self) -> isize {
        match self {
            Dup(..) |
            PushNumber8(..) |
            PushNumber16(..) |
            PushNumber32(..) |
            PushNumber64(..) |
            PushBool(..) |
            PushString(..) |
            PushType(..)
                => 1,
            Pop |
            MoveTo(..) |
            Add(..) |
            Sub(..) |
            Mul(..) |
            Div(..) |
            Mod(..) |
            Equal(..) |
            Inequal(..) |
            Greater(..) |
            GreaterEqual(..) |
            Less(..) |
            LessEqual(..) |
            And |
            Or
                => -1,
            _ => 0
        }
    }
}
