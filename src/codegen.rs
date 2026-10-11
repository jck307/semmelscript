use std::{
    collections::HashMap,
};
use crate::{
    Result,
    instruction::Instruction,
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
        $codegen.set_stack_index(start_stack_index);
        result
    }}
}

macro_rules! replace {
    ($functions:expr, $location:expr, $value:expr) => {{
        $functions[$location.0][$location.1] = $value;
    }}
}

type Location = (usize, usize);

#[derive(Default)]
pub struct CodeGenerator {
    pub builtins: Builtins,
    pub functions_final: Box<[Box<[Instruction]>]>,
    functions: Vec<Vec<Instruction>>,
    current_function_id: usize,
    labels: HashMap<Box<str>, usize>,
    gotos: Vec<(Location, Box<str>)>,
    function_calls: Vec<(Location, usize)>,
    stack_index: usize,
}

#[derive(Default)]
pub struct Scope {
    register: HashMap<Box<str>, usize>,
    parent: *mut Scope,
}

pub trait Generate {
    fn generate(&self, codegen: &mut CodeGenerator, scope: &mut Scope) -> Result<()>;

    fn generate_block(&self, codegen: &mut CodeGenerator, scope: &mut Scope)
            -> Result<()> {
        block!(codegen, scope, {
            self.generate(codegen, scope)?;
        });
        Ok(())
    }
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            functions: vec![vec![]],
            ..Self::default()
        }
    }

    fn instruction_index(&self) -> usize {
        self.current_function().len()
    }

    fn current_function(&self) -> &Vec<Instruction> {
        &self.functions[self.current_function_id]
    }

    fn current_function_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.functions[self.current_function_id]
    }

    fn new_function(&mut self, scope: &mut Scope, ident: &Box<str>) -> usize {
        let old_id = self.current_function_id;
        self.current_function_id = self.functions.len();
        self.functions.push(Vec::new());
        scope.register_function(ident, self.current_function_id);
        old_id
    }

    fn exit_function(&mut self, old_id: usize) {
        self.current_function_id = old_id;
    }

    fn push(&mut self, instr: Instruction) -> Location {
        self.stack_index =
            (self.stack_index as isize + instr.get_stack_diff()) as usize;
        self.current_function_mut().push(instr);
        (self.current_function_id, self.instruction_index() - 1)
    }

    fn push_goto(&mut self, ident: &Box<str>) {
        let index = self.push(Instruction::Placeholder);
        self.gotos.push((index, ident.clone()));
    }

    fn replace(&mut self, location: Location, instr: Instruction) {
        replace!(self.functions, location, instr);
    }

    fn call_function(&mut self, scope: &mut Scope, ident: &Box<str>) -> Result<()> {
        if let Some(builtin) = self.builtins.get(ident) {
            self.push(Instruction::CallInternal(*builtin));
            Ok(())
        } else if let Ok(id) = scope.get_ident(self, ident) {
            let index = self.push(Instruction::Placeholder);
            self.function_calls.push((index, id));
            Ok(())
        } else {
            Err(format!("invalid function: {ident}").into())
        }
    }

    fn set_stack_index(&mut self, index: usize) {
        if self.stack_index != index {
            self.push(Instruction::SetStackIndex(index));
            self.stack_index = index;
        }
    }

    fn register_label(&mut self, ident: &Box<str>) {
        self.labels.insert(ident.clone(), self.instruction_index());
    }

    fn get_label(&self, ident: &Box<str>) -> Result<usize> {
        if let Some(index) = self.labels.get(ident)
            { Ok(*index) } else { Err(format!("invalid label: {ident}").into()) }
    }

    pub fn do_final(&mut self) -> Result<()> {
        for (location, label) in self.gotos.clone() {
            replace!(self.functions, location, Instruction::Goto(self.get_label(&label)?));
        }

        let mut functions_final = Vec::new();
        while self.functions.len() > 0 {
            let function = self.functions.pop().unwrap().into_boxed_slice();
            functions_final.insert(0, function);
        }

        for (location, id) in &self.function_calls {
            let instr = Instruction::Call(functions_final[*id].clone().into());
            replace!(functions_final, location, instr);
        }

        self.functions_final = functions_final.into_boxed_slice();

        for function in &self.functions {
            for instruction in function {
                if let Instruction::Placeholder = instruction {
                    panic!();
                }
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
        self.register.insert(ident.clone(), codegen.stack_index);
    }

    fn register_function(&mut self, ident: &Box<str>, id: usize) {
        self.register.insert(ident.clone(), id);
    }

    fn is_root(&self) -> bool {
        self.parent.is_null()
    }

    fn get_ident(&self, codegen: &mut CodeGenerator, ident: &Box<str>) -> Result<usize> {
        if let Some(value) = self.register.get(ident) {
            Ok(codegen.stack_index - *value - 1)
        } else if !self.is_root() {
            unsafe {
                (*self.parent).get_ident(codegen, ident)
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
                codegen.push(
                    Instruction::PushNumber64(Type::I64, integer.cast_unsigned())
                );
            }
            String(string) => {
                codegen.push(Instruction::PushString(string.to_string())); }
            Boolean(boolean) => { codegen.push(Instruction::PushBool(*boolean)); }
            Identifier(ident) => {
                let rel = scope.get_ident(codegen, ident)?;
                codegen.push(Instruction::Dup(rel)); }
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
                        codegen.call_function(scope, ident)?;
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
                let Node::Block(block) = &**block else { unreachable!() };
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
                    codegen.push(
                        Instruction::PushNumber64(
                            Type::U64,
                            a.try_into().expect("invalid value")
                    ));
                    let start_instr_index = block!(codegen, scope, {
                        let start_instr_index = codegen.instruction_index();
                        block.generate(codegen, scope)?;
                        start_instr_index
                    });
                    codegen.push(
                        Instruction::LoopUpdate(
                            b.try_into().expect("invalid value"),
                            1,
                            start_instr_index
                    ));
                    return Ok(())
                }}
                todo!("can only iterate over constant ranges, not {iterable:?}")
            }
            DefineFunction(ident, _args, block) => {
                let old_id = codegen.new_function(scope, ident);
                block.generate(codegen, &mut Scope::default())?;
                codegen.exit_function(old_id);
            }
            other => todo!("{other:?}")
        }
        Ok(())
    }
}

impl Generate for Block {
    fn generate(&self, codegen: &mut CodeGenerator, scope: &mut Scope) -> Result<()> {
        for node in &self.nodes {
            let start_stack_index = codegen.stack_index;
            node.generate(codegen, scope)?;
            match node {
                Node::DefineVariable(..) => {}
                _ => {
                    codegen.set_stack_index(start_stack_index);
                }
            }
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
                let rel = scope.get_ident(codegen, ident)?;
                codegen.push(Instruction::MoveTo(rel));
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
