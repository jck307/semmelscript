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
            Block(block) => {
                for node in &mut block.nodes {
                    node.traverse(func)?;
                }
            }
            ParenArgs(callable, args) => {
                callable.traverse(func)?;
                for arg in args {
                    arg.traverse(func)?;
                }
            }
            BinaryOp(bin_op) => {
                bin_op.a.traverse(func)?;
                bin_op.b.traverse(func)?;
            }
            DefineVariable(_, value) => value.traverse(func)?,
            DefineFunction(_, _, block) => block.traverse(func)?,
            If(condition, block, ext) => {
                condition.traverse(func)?;
                block.traverse(func)?;
                if let Some(ext) = ext {
                    ext.traverse(func)?;
                }
            }
            For(_, iterable, block) => {
                iterable.traverse(func)?;
                block.traverse(func)?;
            }
            While(condition, block) => {
                condition.traverse(func)?;
                block.traverse(func)?;
            }
            List(args) => {
                for arg in args {
                    arg.traverse(func)?;
                }
            }
            _ => {}
        }
        func(self, End)?;
        Ok(())
    }
}
