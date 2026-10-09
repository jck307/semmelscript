use std::collections::HashMap;
use crate::{
    runtime::{Instruction, Type},
    node::*,
    token::Operator,
    stdlib::*,
};

#[derive(Default)]
pub struct CodeGenerator {
    pub instructions: Vec<Instruction>,
    pub stack_index: usize,
    pub stack_register: HashMap<Box<str>, usize>,
    pub labels: HashMap<Box<str>, usize>,
    pub builtins: Builtins,
}

impl CodeGenerator {
    fn push(&mut self, instr: Instruction) -> usize {
        self.instructions.push(instr);
        self.instructions.len() - 1
    }

    fn replace(&mut self, index: usize, instr: Instruction) {
        self.instructions[index] = instr;
    }

    fn register(&mut self, ident: &Box<str>) {
        self.stack_register.insert(ident.clone(), self.stack_index);
        self.stack_index += 1;
    }

    fn register_label(&mut self, ident: &Box<str>) {
        self.labels.insert(ident.clone(), self.instructions.len());
    }

    fn get_ident(&self, ident: &Box<str>) -> usize {
        *self.stack_register.get(ident).expect(&*format!("invalid identifier: {ident}"))
    }

    fn get_builtin(&self, ident: &Box<str>) -> Builtin {
        *self.builtins.get(ident).expect(&*format!("invalid builtin: {ident}"))
    }

    fn get_label(&self, ident: &Box<str>) -> usize {
        *self.labels.get(ident).expect(&*format!("invalid label: {ident}"))
    }

    fn get_index(&self) -> usize {
        self.instructions.len()
    }
}

pub trait Generate {
    fn generate(&self, codegen: &mut CodeGenerator);
}

impl Generate for Node {
    fn generate(&self, codegen: &mut CodeGenerator) {
        use Node::*;
        match self {
            Block(block) => block.generate(codegen),
            BinaryOp(bin_op) => bin_op.generate(codegen),
            Label(ident) => codegen.register_label(ident),
            Goto(ident) => {
                codegen.push(Instruction::Goto(codegen.get_label(ident)));
            }
            Integer(integer) => {
                codegen.push(Instruction::PushNumber32(Type::I32, integer.cast_unsigned()));
            }
            String(string) => {
                codegen.push(Instruction::PushString(string.to_string()));
            }
            Identifier(ident) => {
                codegen.push(Instruction::DupFrom(codegen.get_ident(ident)));
            }
            DefineVariable(ident, node) => {
                codegen.register(ident);
                node.generate(codegen);
            }
            ParenArgs(callable, args) => {
                for arg in args {
                    arg.generate(codegen);
                }
                match &**callable {
                    Identifier(ident) => {
                        let func = codegen.get_builtin(ident);
                        codegen.push(Instruction::CallInternal(func));
                    }
                    _ => panic!()
                }
            }
            If(condition, block, _ext) => {
                condition.generate(codegen);
                let placeholder_index = codegen.push(Instruction::Placeholder);
                block.generate(codegen);
                codegen.replace(
                    placeholder_index,
                    Instruction::GotoConditional(codegen.get_index())
                );
            }
            other => todo!("{}", format!("{other:?}"))
        }
    }
}

impl Generate for Block {
    fn generate(&self, codegen: &mut CodeGenerator) {
        for node in &self.nodes {
            node.generate(codegen);
        }
    }
}

impl Generate for BinaryOp {
    fn generate(&self, codegen: &mut CodeGenerator) {
        match &self.op {
            Operator::Assign => {
                self.b.generate(codegen);
                let Node::Identifier(ref ident) = self.a else { panic!() };
                codegen.push(Instruction::MoveTo(codegen.get_ident(ident)));
                return
            }
            // Operator::AddAssign |
            // Operator::SubAssign |
            // Operator::MulAssign |
            // Operator::DivAssign |
            // Operator::PowAssign |
            // Operator::ModAssign => 
            _ => {}
        }

        self.a.generate(codegen);
        self.b.generate(codegen);
        let ty = Type::I32; // TODO ...
        codegen.push(match &self.op {
            Operator::Add => Instruction::Add(ty),
            Operator::Sub => Instruction::Sub(ty),
            Operator::Mul => Instruction::Mul(ty),
            Operator::Div => Instruction::Div(ty),
            Operator::Mod => Instruction::Mod(ty),
            Operator::Equal => Instruction::Equal(ty),
            Operator::Inequal => Instruction::Inequal(ty),
            Operator::Less => Instruction::Less(ty),
            Operator::LessEqual => Instruction::LessEqual(ty),
            Operator::Greater => Instruction::Greater(ty),
            Operator::GreaterEqual => Instruction::GreaterEqual(ty),
            Operator::And => Instruction::And,
            Operator::Or => Instruction::Or,
            other => todo!("{}", format!("{other:?}"))
        });
    }
}
