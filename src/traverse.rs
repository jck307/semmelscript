use crate::{
    *,
    node::*,
};

pub enum TraverseMode {
    Start,
    End,
}

use TraverseMode::*;

impl Node {
    pub fn traverse<T>(&mut self, func: &T) -> Result<()>
            where T: Fn(&mut Node, TraverseMode) -> Result<()> {
        use Node::*;
        func(self, Start)?;
        match self {
            Block(block, _) => {
                for node in &mut block.nodes {
                    node.traverse(func)?;
                }
            }
            ParenArgs(callable, args, _) => {
                callable.traverse(func)?;
                for arg in args {
                    arg.traverse(func)?;
                }
            }
            BinaryOp(bin_op, _) => {
                bin_op.a.traverse(func)?;
                bin_op.b.traverse(func)?;
            }
            UnaryOp(un_op, _) => {
                un_op.value.traverse(func)?;
            }
            DefineVariable(_, value, _) => value.traverse(func)?,
            DefineFunction(_, _, block, _) => block.traverse(func)?,
            If(statements, end, _) => {
                for (condition, block) in statements {
                    condition.traverse(func)?;
                    block.traverse(func)?;
                }

                if let Some(block) = end {
                    block.traverse(func)?;
                }
            }
            For(_, iterable, block, _) => {
                iterable.traverse(func)?;
                block.traverse(func)?;
            }
            While(condition, block, _) => {
                condition.traverse(func)?;
                block.traverse(func)?;
            }
            List(args, _) => {
                for arg in args {
                    arg.traverse(func)?;
                }
            }
            Label(..) |
            Goto(..) |
            Identifier(..) |
            String(..) |
            Integer(..) |
            Float(..) |
            Boolean(..)
                => {}
        }
        func(self, End)?;
        Ok(())
    }
}
