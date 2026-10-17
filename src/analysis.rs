use {
    std::{
        collections::HashMap,
        rc::Rc,
        cell::RefCell,
    },
    crate::{
        Result,
        debug::*,
        traverse::TraverseMode,
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
    step1::execute(debug_log, block, initial_scope, analysis.clone())?;
    let analysis = analysis.take();
    Ok(analysis)
}

mod step1 {
    use super::*;

    #[derive(Default, Debug)]
    struct State {
        scope: Vec<(usize, Vec<HashMap<Box<str>, usize>>)>,
        scope_added: bool,
    }

    pub fn execute(
            #[allow(unused_variables)]
            debug_log: DebugLog,
            block: &mut Node,
            initial_scope: HashMap<Box<str>, usize>,
            analysis: Rc<RefCell<Analysis>>) -> Result<()> {

        let mut state = State::default();
        state.scope.push((0, vec![initial_scope]));
        let state = Rc::new(RefCell::new(state));

        {
            block.traverse(
                &move |node: &mut Node, mode| {
                    use TraverseMode::*;
                    let mut state = (*state).borrow_mut();
                    let mut analysis = (*analysis).borrow_mut();

                    let last_func = state.scope.len()-1;

                    if let Node::Block(..) = node {
                        match (&mode, state.scope_added) {
                            (Start, false) => {
                                state.scope[last_func].1.push(HashMap::new());
                            }
                            (Start, true) => {
                                state.scope_added = false;
                            }
                            (End, _) => {
                                state.scope[last_func].1.pop();
                            }
                        }
                    }

                    if let Node::DefineFunction(..) = node && let End = mode {
                        state.scope.pop();
                    }

                    if let Start = mode {
                        match node {
                            Node::DefineVariable(ident, ..) |
                            Node::DefineFunction(ident, ..) |
                            Node::For(ident, ..) => {
                                let last_block = state.scope[last_func].1.len()-1;
                                let id = analysis.next_id();
                                {
                                    let ident = ident.as_string().clone();
                                    debugln!(debug_log, "assign id: {} -> {}", ident, id);
                                    state.scope[last_func].1[last_block].insert(ident, id);
                                }
                                *ident = Identifier::Id(id);

                                match node {
                                    Node::DefineFunction(_, args, block) => {
                                        let mut args_scope = HashMap::new();
                                        let mut sig = FunctionSignature::default();
                                        for (i, ident) in args.iter().enumerate() {
                                            let ident = ident.as_string().clone();
                                            let id = analysis.next_id();
                                            args_scope.insert(ident, id);
                                            sig.0.push(Type::Unknown);
                                            analysis.local_ids.insert(id, i);
                                            analysis.types.insert(id, Type::Unknown);
                                        }
                                        let Node::Block(block) = &**block else { panic!() };
                                        if block.return_last {
                                            sig.1.push(Type::Unknown);
                                        }
                                        state.scope.push((args.len(), vec![args_scope]));
                                        state.scope_added = true;
                                        analysis.types.insert(id, Type::Function(sig));
                                    }
                                    _ => {
                                        let local_id = state.scope[last_func].0;
                                        state.scope[last_func].0 += 1;
                                        analysis.local_ids.insert(id, local_id);
                                        analysis.types.insert(id, Type::Unknown);
                                    }
                                }
                            }
                            Node::Identifier(ident) => {
                                for idents in state.scope[last_func].1.iter() {
                                    if let Some(id) = idents.get(ident.as_string()) {
                                        *ident = Identifier::Id(*id);
                                        return Ok(())
                                    }
                                }
                                panic!("name error: {}", ident.as_string());
                            }
                            _ => {}
                        }
                    }

                    Ok(())
                },
            )?;
        }

        Ok(())
    }
}
