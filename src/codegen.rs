use std::collections::HashMap;
use crate::{
    runtime::{Instruction, Type},
    node::*,
    token::Operator,
    stdlib::*,
};

macro_rules! block {
    ($codegen:ident, {$($tt:tt)*}) => {
        let start_stack_index = $codegen.stack_index;
        let start_depth = $codegen.block_depth;
        $codegen.block_depth += 1;
        $($tt)*
        $codegen.block_depth -= 1;
        $codegen.push(Instruction::SetStackIndex(start_stack_index));
        $codegen.stack_index = start_stack_index;
        for (key, (_, depth)) in $codegen.stack_register.clone() {
            if start_depth < depth {
                $codegen.stack_register.remove(&key);
            }
        }
    }
}

#[derive(Default)]
pub struct CodeGenerator {
    pub instructions: Vec<Instruction>,
    pub builtins: Builtins,
    pub stack_index: usize,
    pub stack_register: HashMap<Box<str>, (usize, u16)>,
    pub block_depth: u16,
    pub labels: HashMap<Box<str>, usize>,
    pub gotos: Vec<(usize, Box<str>)>,
}

impl CodeGenerator {
    fn instruction_index(&self) -> usize {
        self.instructions.len()
    }

    fn push(&mut self, instr: Instruction) -> usize {
        self.instructions.push(instr);
        self.instructions.len() - 1
    }

    fn push_goto(&mut self, ident: &Box<str>) {
        let index = self.push(Instruction::Placeholder);
        self.gotos.push((index, ident.clone()));
    }

    fn replace(&mut self, index: usize, instr: Instruction) {
        self.instructions[index] = instr;
    }

    fn register(&mut self, ident: &Box<str>) {
        self.stack_register.insert(ident.clone(), (self.stack_index, self.block_depth));
        self.stack_index += 1;
    }

    fn register_label(&mut self, ident: &Box<str>) {
        self.labels.insert(ident.clone(), self.instruction_index());
    }

    fn get_ident(&self, ident: &Box<str>) -> usize {
        (*self.stack_register.get(ident).expect(&*format!("invalid identifier: {ident}"))).0
    }

    fn get_builtin(&self, ident: &Box<str>) -> Builtin {
        *self.builtins.get(ident).expect(&*format!("invalid builtin: {ident}"))
    }

    fn get_label(&self, ident: &Box<str>) -> usize {
        *self.labels.get(ident).expect(&*format!("invalid label: {ident}"))
    }

    pub fn do_final(&mut self) {
        for (index, label) in self.gotos.clone() {
            self.replace(index, Instruction::Goto(self.get_label(&label)));
        }

        for instruction in &self.instructions {
            if let Instruction::Placeholder = instruction {
                panic!();
            }
        }
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
            Goto(ident) => { codegen.push_goto(ident); }
            Integer(integer) => {
                codegen.push(Instruction::PushNumber64(Type::I64, integer.cast_unsigned()));
            }
            String(string) => {
                codegen.push(Instruction::PushString(string.to_string())); }
            Boolean(boolean) => { codegen.push(Instruction::PushBool(*boolean)); }
            Identifier(ident) => {
                codegen.push(Instruction::DupFrom(codegen.get_ident(ident))); }
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
            If(condition, block, ext) => {
                condition.generate(codegen);
                let placeholder_index = codegen.push(Instruction::Placeholder);
                block!(codegen, {
                    block.generate(codegen);
                });
                codegen.replace(
                    placeholder_index,
                    Instruction::GotoIf(codegen.instruction_index())
                );
                if let Some(ext) = ext {
                    block!(codegen, {
                        match &**ext {
                            Block(block) => block.generate(codegen),
                            If(..) => ext.generate(codegen),
                            _ => unreachable!()
                        }
                    });
                }
            }
            For(ident, iterable, block) => {
                use Operator::*;
                'block: { if let Node::BinaryOp(op) = &**iterable {
                    match op.op {
                        RangeExcl | RangeIncl => {}
                        _ => { break 'block }
                    }
                    let (a, b) = match (op.a.clone(), op.b.clone()) {
                        (Node::Integer(a), Node::Integer(b)) => (a, b),
                        _ => { break 'block }
                    };
                    let b = if op.op == RangeIncl { b + 1 } else { b };
                    codegen.register(ident);
                    block!(codegen, {
                        codegen.push(Instruction::PushNumber64(
                            Type::U64,
                            a.try_into().expect("invalid value")
                        ));
                        let start_instr_index = codegen.instruction_index();
                        block.generate(codegen);
                    });
                    codegen.push(Instruction::LoopUpdate(
                        b.try_into().expect("invalid value"),
                        1,
                        start_instr_index
                    ));
                    return
                }}
                todo!("can only iterate over constant ranges, not {iterable:?}")
            }
            other => todo!("{other:?}")
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
        let ty = Type::I64;
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
            other => todo!("{other:?}")
        });
    }
}
