use {
    std::{
        collections::HashMap,
        rc::Rc,
        cell::RefCell,
    },
    crate::{
        Result,
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
        block: &mut Node,
        initial_scope: HashMap<Box<str>, usize>,
        analysis: Analysis) -> Result<Analysis> {

    let analysis = Rc::new(RefCell::new(analysis));
    step1::execute(block, initial_scope, analysis.clone())?;
    let analysis = analysis.take();
    Ok(analysis)
}

mod step1 {
    use super::*;

    #[derive(Default, Debug)]
    struct State {
        scope: Vec<(usize, HashMap<Box<str>, usize>)>,
        function: bool,
    }

    pub fn execute(
            block: &mut Node,
            initial_scope: HashMap<Box<str>, usize>,
            analysis: Rc<RefCell<Analysis>>) -> Result<()> {

        let mut state = State::default();
        state.scope.push((0, initial_scope));
        let state = Rc::new(RefCell::new(state));

        {
            block.traverse(
                &move |node: &mut Node, mode| {
                    use TraverseMode::*;
                    let mut state = (*state).borrow_mut();
                    let mut analysis = (*analysis).borrow_mut();

                    if let Node::Block(..) = node {
                        match (&mode, state.function) {
                            (Start, false) => {
                                state.scope.push((0, HashMap::new()));
                            }
                            (Start, true) => {
                                state.function = false;
                            }
                            (End, _) => {
                                state.scope.pop();
                            }
                        }
                    }

                    if let Start = mode {
                        match node {
                            Node::DefineVariable(ident, ..) |
                            Node::DefineFunction(ident, ..) |
                            Node::For(ident, ..) => {
                                let id = analysis.next_id();
                                let last = state.scope.len()-1;
                                {
                                    let ident = ident.as_string().clone();
                                    state.scope[last].1.insert(ident, id);
                                }
                                *ident = Identifier::Id(id);

                                match node {
                                    Node::DefineFunction(_, args, _) => {
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
                                        state.scope.push((args.len(), args_scope));
                                        state.function = true;
                                        analysis.types.insert(id, Type::Function(sig));
                                    }
                                    _ => {
                                        let local_id = state.scope[last].0;
                                        state.scope[last].0 += 1;
                                        analysis.local_ids.insert(id, local_id);
                                        analysis.types.insert(id, Type::Unknown);
                                    }
                                }
                            }
                            Node::Identifier(ident) => {
                                for (_, idents) in state.scope.iter() {
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
