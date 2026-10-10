use std::collections::HashMap;
use crate::{
    Result,
    runtime::Instruction,
    types::Type,
    token::Operator,
    node::*,
    stdlib::*,
};

macro_rules! block {
    ($codegen:ident, $scope:ident, {$($tt:tt)*}) => {{
        let start_stack_index = $codegen.stack_index;
        let result = {
            let $scope = &mut $scope.descend();
            $($tt)*
        };
        $codegen.push(Instruction::SetStackIndex(start_stack_index));
        $codegen.stack_index = start_stack_index;
        result
    }}
}

#[derive(Default)]
pub struct CodeGenerator {
    pub instructions: Vec<Instruction>,
    pub builtins: Builtins,
    pub stack_index: usize,
    pub labels: HashMap<Box<str>, usize>,
    pub gotos: Vec<(usize, Box<str>)>,
}

#[derive(Default)]
pub struct Scope {
    pub stack_register: HashMap<Box<str>, usize>,
    pub parent: *mut Scope,
}

pub trait Generate {
    fn generate(&self, codegen: &mut CodeGenerator, scope: &mut Scope) -> Result<()> ;

    fn generate_block(&self, codegen: &mut CodeGenerator, scope: &mut Scope)
            -> Result<()> {
        block!(codegen, scope, {
            self.generate(codegen, scope)?;
        });
        Ok(())
    }
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

    fn get_builtin(&self, ident: &Box<str>) -> Result<Builtin> {
        if let Some(builtin) = self.builtins.get(ident)
            { Ok(*builtin) } else { Err(format!("invalid builtin: {ident}").into()) }
    }

    fn register_label(&mut self, ident: &Box<str>) {
        self.labels.insert(ident.clone(), self.instruction_index());
    }

    fn get_label(&self, ident: &Box<str>) -> Result<usize> {
        if let Some(index) = self.labels.get(ident)
            { Ok(*index) } else { Err(format!("invalid label: {ident}").into()) }
    }

    pub fn do_final(&mut self) -> Result<()> {
        for (index, label) in self.gotos.clone() {
            self.replace(index, Instruction::Goto(self.get_label(&label)?));
        }
        for instruction in &self.instructions {
            if let Instruction::Placeholder = instruction {
                panic!();
            }
        }
        Ok(())
    }
}

impl Scope {
    fn descend(&mut self) -> Self {
        Self {
            parent: self,
            ..Self::default()
        }
    }

    fn register(&mut self, codegen: &mut CodeGenerator, ident: &Box<str>) {
        self.stack_register.insert(ident.clone(), codegen.stack_index);
        codegen.stack_index += 1;
    }

    fn is_root(&self) -> bool {
        self.parent.is_null()
    }

    fn get_ident(&self, ident: &Box<str>) -> Result<usize> {
        if let Some(value) = self.stack_register.get(ident) {
            Ok(*value)
        } else if !self.is_root() {
            unsafe {
                (*self.parent).get_ident(ident)
            }
        } else {
            Err(format!("invalid identifier: {ident}").into())
        }
    }
}

impl Generate for Node {
    fn generate(&self, codegen: &mut CodeGenerator, scope: &mut Scope) -> Result<()> {
        use Node::*;
        match self {
            Block(block) => block.generate_block(codegen, scope)?,
            BinaryOp(bin_op) => bin_op.generate(codegen, scope)?,
            Label(ident) => codegen.register_label(ident),
            Goto(ident) => { codegen.push_goto(ident); }
            Integer(integer) => {
                codegen.push(Instruction::PushNumber64(Type::I64, integer.cast_unsigned()));
            }
            String(string) => {
                codegen.push(Instruction::PushString(string.to_string())); }
            Boolean(boolean) => { codegen.push(Instruction::PushBool(*boolean)); }
            Identifier(ident) => {
                codegen.push(Instruction::DupFrom(scope.get_ident(ident)?)); }
            DefineVariable(ident, node) => {
                scope.register(codegen, ident);
                node.generate(codegen, scope)?;
            }
            ParenArgs(callable, args) => {
                for arg in args {
                    arg.generate(codegen, scope)?;
                }
                match &**callable {
                    Identifier(ident) => {
                        let func = codegen.get_builtin(ident)?;
                        codegen.push(Instruction::CallInternal(func));
                    }
                    _ => panic!()
                }
            }
            If(condition, block, ext) => {
                condition.generate(codegen, scope)?;
                let placeholder_index = codegen.push(Instruction::Placeholder);
                block.generate_block(codegen, scope)?;
                codegen.replace(
                    placeholder_index,
                    Instruction::GotoIf(codegen.instruction_index())
                );
                if let Some(ext) = ext {
                    match &**ext {
                        Block(block) => block.generate_block(codegen, scope)?,
                        If(..) => ext.generate(codegen, scope)?,
                        _ => unreachable!()
                    }
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
                    scope.register(codegen, ident);
                    let start_instr_index = block!(codegen, scope, {
                        codegen.push(Instruction::PushNumber64(
                            Type::U64,
                            a.try_into().expect("invalid value")
                        ));
                        let start_instr_index = codegen.instruction_index();
                        block.generate(codegen, scope)?;
                        start_instr_index
                    });
                    codegen.push(Instruction::LoopUpdate(
                        b.try_into().expect("invalid value"),
                        1,
                        start_instr_index
                    ));
                    return Ok(())
                }}
                todo!("can only iterate over constant ranges, not {iterable:?}")
            }
            // DefineFunction(ident, args, block) => {

            // }
            other => todo!("{other:?}")
        }
        Ok(())
    }
}

impl Generate for Block {
    fn generate(&self, codegen: &mut CodeGenerator, scope: &mut Scope) -> Result<()> {
        for node in &self.nodes {
            node.generate(codegen, scope)?;
        }
        Ok(())
    }
}

impl Generate for BinaryOp {
    fn generate(&self, codegen: &mut CodeGenerator, scope: &mut Scope) -> Result<()> {
        match &self.op {
            Operator::Assign => {
                self.b.generate(codegen, scope)?;
                let Node::Identifier(ref ident) = self.a else { panic!() };
                codegen.push(Instruction::MoveTo(scope.get_ident(ident)?));
                return Ok(())
            }
            // Operator::AddAssign |
            // Operator::SubAssign |
            // Operator::MulAssign |
            // Operator::DivAssign |
            // Operator::PowAssign |
            // Operator::ModAssign => 
            _ => {}
        }

        self.a.generate(codegen, scope)?;
        self.b.generate(codegen, scope)?;
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
        Ok(())
    }
}
