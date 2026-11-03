mod assign_ids;
mod type_checking;

use {
    std::{
        collections::HashMap,
        rc::Rc,
        cell::RefCell,
    },
    crate::{
        Result,
        debug::*,
        traverse::TraverseMode::*,
        types::{Type, FunctionSignature},
        node::{Node, Identifier},
    },
};

#[derive(Default, Debug)]
pub struct Analysis {
    pub types: HashMap<usize, Type>,
    pub local_ids: HashMap<usize, usize>,
    next_id: usize,
}

impl Analysis {
    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub fn execute(
        debug_log: DebugLog,
        block: &mut Node,
        initial_scope: HashMap<Box<str>, usize>,
        analysis: Analysis) -> Result<Analysis> {

    let analysis = Rc::new(RefCell::new(analysis));
    assign_ids::execute(debug_log.clone(), block, initial_scope, analysis.clone())?;
    type_checking::execute(debug_log, block, analysis.clone())?;
    let analysis = analysis.take();
    Ok(analysis)
}
