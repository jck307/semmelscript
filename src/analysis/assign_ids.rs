use super::*;

#[derive(Default, Debug)]
struct State {
    scope: Vec<(usize, HashMap<Box<str>, usize>)>,
    scope_added: bool,
}

pub fn execute(
        #[allow(unused_variables)]
        debug_log: DebugLog,
        block: &mut Node,
        initial_scope: HashMap<Box<str>, usize>,
        analysis: Rc<RefCell<Analysis>>) -> Result<()> {

    let mut state = State::default();
    state.scope.push((0, initial_scope));
    let state = Rc::new(RefCell::new(state));

    block.traverse(
        &move |node: &mut Node, mode| {
            let mut state = (*state).borrow_mut();
            let mut analysis = (*analysis).borrow_mut();

            let last = state.scope.len()-1;

            if let Node::Block(..) = node {
                match (&mode, state.scope_added) {
                    (Start, false) => {
                        let local_id = state.scope[last].0;
                        state.scope.push((local_id, HashMap::new()));
                    }
                    (Start, true) => {
                        state.scope_added = false;
                    }
                    (End, _) => {
                        state.scope.pop();
                    }
                }
            }

            // if let Node::DefineFunction(..) = node && let End = mode {
            //     state.scope.pop();
            // }

            let last = state.scope.len()-1;

            match (mode, &mut *node) {
                (End, Node::DefineVariable(ident, ..)) |
                (Start, Node::DefineFunction(ident, ..)) |
                (Start, Node::For(ident, ..)) => {
                    let id = analysis.next_id();
                    {
                        let ident = ident.as_string().clone();
                        debugln!(debug_log, "assign id: {} -> {}", ident, id);
                        state.scope[last].1.insert(ident, id);
                    }
                    *ident = Identifier::Id(id);

                    match node {
                        Node::DefineFunction(_, args, return_type, block, _) => {
                            let mut args_scope = HashMap::new();
                            let mut sig = FunctionSignature::default();
                            for (i, (ident, ty)) in args.iter().enumerate() {
                                let ident = ident.as_string().clone();
                                let id = analysis.next_id();
                                debugln!(debug_log, "assign id: {} -> {}", ident, id);
                                args_scope.insert(ident, id);
                                sig.0.push(ty.clone());
                                analysis.local_ids.insert(id, i);
                                analysis.types.insert(id, ty.clone());
                            }
                            let Node::Block(block, _) = &**block
                                else { panic!() };
                            if return_type.len() != 0 {
                                if !block.return_last {
                                    return Err("expected return value".into())
                                } else {
                                    sig.1 = return_type.clone();
                                }
                            }
                            state.scope.push((args.len(), args_scope));
                            state.scope_added = true;
                            analysis.types.insert(
                                id,
                                Type::Function(Box::new(sig))
                            );
                        }
                        _ => {
                            let local_id = state.scope[last].0;
                            state.scope[last].0 += 1;
                            analysis.local_ids.insert(id, local_id);
                            analysis.types.insert(id, Type::Unknown);
                        }
                    }
                }
                (Start, Node::Identifier(ident, _)) => {
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

            Ok(())
        },
    )?;

    Ok(())
}
