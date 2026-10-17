use {
    std::{
        collections::HashMap,
    },
    crate::{
        Result,
        instruction::Instruction,
        types::Type,
        token::Operator,
        debug::*,
        analysis::Analysis,
        node::{Node, Block, BinaryOp},
    },
};

macro_rules! block {
    ($codegen:ident, $block:expr, {$($tt:tt)*}) => {{
        let start_stack_index = $codegen.stack_index;
        let result = { $($tt)* };
        $codegen.pop(
            $codegen.stack_index - start_stack_index
            - if $block.return_last { 1 } else { 0 }
        );
        result
    }}
}

type Location = (usize, usize);

pub struct CodeGenerator {
    pub functions: HashMap<usize, Vec<Instruction>>,
    current_function_id: usize,
    labels: HashMap<Box<str>, usize>,
    gotos: Vec<(Location, Box<str>)>,
    function_calls: Vec<(Location, usize)>,
    stack_index: usize,
    analysis: Analysis,
    #[allow(unused)]
    debug_log: DebugLog,
}

pub trait Generate {
    fn generate(&self, codegen: &mut CodeGenerator) -> Result<()>;

    // fn generate_block(&self, codegen: &mut CodeGenerator)
    //         -> Result<()> {
    //     block!(codegen, self, {
    //         self.generate(codegen)?;
    //     });
    //     Ok(())
    // }
}

impl CodeGenerator {
    pub fn new(
            debug_log: DebugLog,
            functions: HashMap<usize, Vec<Instruction>>,
            analysis: Analysis) -> Self {

        Self {
            debug_log,
            functions,
            analysis,
            current_function_id: 0,
            labels: HashMap::new(),
            gotos: Vec::new(),
            function_calls: Vec::new(),
            stack_index: 0,
        }
    }

    fn instruction_index(&self) -> usize {
        self.current_function().len()
    }

    fn current_function(&self) -> &Vec<Instruction> {
        &self.functions[&self.current_function_id]
    }

    fn current_function_mut(&mut self) -> &mut Vec<Instruction> {
        self.functions.get_mut(&self.current_function_id).unwrap()
    }

    fn new_function(&mut self, id: usize) -> usize {
        let old_id = self.current_function_id;
        self.current_function_id = id;
        self.functions.insert(id, Vec::new());
        old_id
    }

    fn exit_function(&mut self, old_id: usize) {
        self.current_function_id = old_id;
    }

    fn get_rel_id(&self, id: &usize) -> usize {
        self.stack_index - self.analysis.local_ids[id] - 1
    }

    fn push(&mut self, instr: Instruction) -> Location {
        self.stack_index =
            (self.stack_index as isize + instr.get_stack_diff()) as usize;
        if self.current_function_id == 0 {
            debugln!(self.debug_log, "{instr:?}: {}", self.stack_index);
        }
        self.current_function_mut().push(instr);
        (self.current_function_id, self.instruction_index() - 1)
    }

    fn push_goto(&mut self, ident: &Box<str>) {
        let index = self.push(Instruction::Placeholder);
        self.gotos.push((index, ident.clone()));
    }

    fn replace(&mut self, location: Location, instr: Instruction) {
        self.functions.get_mut(&location.0).unwrap()[location.1] = instr;
    }

    fn pop(&mut self, count: usize) {
        if count != 0 {
            self.push(Instruction::Pop(count));
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
            self.replace(location, Instruction::Goto(self.get_label(&label)?));
        }

        for (location, id) in self.function_calls.clone() {
            // TODO fix!!!
            self.replace(location, Instruction::Call(self.functions[&id].clone().into()));
        }

        for (_, instructions) in &self.functions {
            for instruction in instructions {
                if let Instruction::Placeholder = instruction {
                    panic!();
                }
            }
        }

        Ok(())
    }
}

impl Generate for Node {
    fn generate(&self, codegen: &mut CodeGenerator) -> Result<()> {
        use Node::*;
        match self {
            Block(block) => {
                block!(codegen, block, {
                    block.generate(codegen)?;
                });
            }
            BinaryOp(bin_op) => bin_op.generate(codegen)?,
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
            Identifier(id) => {
                codegen.push(Instruction::Dup(codegen.get_rel_id(id.as_id())));
            }
            DefineVariable(_, node) => { node.generate(codegen)?; }
            ParenArgs(callable, args) => {
                let start_stack_index = codegen.stack_index;
                for arg in args {
                    arg.generate(codegen)?;
                }
                // TODO check function return type
                codegen.stack_index = start_stack_index + 1;
                match &**callable {
                    Identifier(id) => {
                        let id = id.as_id();
                        let index = codegen.push(Instruction::Placeholder);
                        codegen.function_calls.push((index, *id));
                    }
                    _ => panic!()
                }
            }
            If(condition, block, ext) => {
                condition.generate(codegen)?;
                let placeholder_index = codegen.push(Instruction::Placeholder);
                let Block(block) = &**block else { panic!() };
                block!(codegen, block, {
                    block.generate(codegen)?;
                });
                codegen.replace(
                    placeholder_index,
                    Instruction::GotoIf(codegen.instruction_index())
                );
                if let Some(ext) = ext {
                    match &**ext {
                        Block(block) => {
                        block!(codegen, block, {
                            block.generate(codegen)?;
                        });
                        }
                        If(..) => ext.generate(codegen)?,
                        _ => unreachable!()
                    }
                }
            }
            For(_ident, iterable, block) => {
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
                    codegen.push(
                        Instruction::PushNumber64(
                            Type::U64,
                            a.try_into().expect("invalid value")
                    ));
                    let start_instr_index = block!(codegen, block, {
                        let start_instr_index = codegen.instruction_index();
                        block.generate(codegen)?;
                        start_instr_index
                    });
                    codegen.stack_index -= 1;
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
            DefineFunction(id, args, block) => {
                let Node::Block(block) = &**block else { panic!() };
                let start_stack_index = codegen.stack_index;
                let old_id = codegen.new_function(*id.as_id());
                codegen.stack_index = args.len();
                {
                    block.generate(codegen)?;
                    if block.return_last {
                        codegen.push(Instruction::RotateRight(args.len() + 1));
                    }
                    let popcount = 
                        if block.return_last { codegen.stack_index.saturating_sub(1) }
                        else { codegen.stack_index };
                    codegen.pop(popcount);
                }
                codegen.exit_function(old_id);
                codegen.stack_index = start_stack_index;
            }
            other => todo!("{other:?}")
        }
        Ok(())
    }
}

impl Generate for Block {
    fn generate(&self, codegen: &mut CodeGenerator) -> Result<()> {
        for (_i, node) in self.nodes.iter().enumerate() {
            let _start_stack_index = codegen.stack_index;
            node.generate(codegen)?;
            match node {
                Node::DefineVariable(..) |
                Node::DefineFunction(..)
                    => {}
                _ => {
                    // if i != self.nodes.len() - 1 {
                    //     codegen.pop(codegen.stack_index - start_stack_index);
                    // }
                }
            }
        }
        Ok(())
    }
}

impl Generate for BinaryOp {
    fn generate(&self, codegen: &mut CodeGenerator) -> Result<()> {
        match &self.op {
            Operator::Assign => {
                self.b.generate(codegen)?;
                let Node::Identifier(ref id) = self.a else { panic!() };
                codegen.push(Instruction::MoveTo(codegen.get_rel_id(id.as_id())));
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

        self.a.generate(codegen)?;
        self.b.generate(codegen)?;
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
