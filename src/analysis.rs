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
    step1::execute(debug_log.clone(), block, initial_scope, analysis.clone())?;
    step2::execute(debug_log, block, analysis.clone())?;
    let analysis = analysis.take();
    Ok(analysis)
}

mod step1 {
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
                                for (i, ident) in args.iter().enumerate() {
                                    let ident = ident.as_string().clone();
                                    let id = analysis.next_id();
                                    args_scope.insert(ident, id);
                                    sig.0.push(Type::Unknown);
                                    analysis.local_ids.insert(id, i);
                                    analysis.types.insert(id, Type::Unknown);
                                }
                                let Node::Block(block, _) = &**block
                                    else { panic!() };
                                if *return_type != Type::Null {
                                    if !block.return_last {
                                        return Err("expected return value".into())
                                    } else {
                                        sig.1.push(return_type.clone());
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
}

mod step2 {
    use super::*;

    enum TypeSet {
        Boolean,
        Integer,
        Float,
        Other,
    }

    pub fn execute(
            #[allow(unused_variables)]
            debug_log: DebugLog,
            block: &mut Node,
            analysis: Rc<RefCell<Analysis>>) -> Result<()> {

        block.traverse(
            &move |node: &mut Node, mode| {
                use crate::types::{NUMERIC_TYPES, INTEGER_TYPES};
                use crate::token::OperatorType::*;

                let mut analysis = (*analysis).borrow_mut();

                if let End = mode {
                    match node {
                        Node::BinaryOp(bin_op, meta) => {
                            let op_ty = bin_op.op.ty();
                            let a_ty = &bin_op.a.meta().ty;
                            let b_ty = &bin_op.b.meta().ty;

                            if op_ty == Assignment {
                                return if let Node::Identifier(..) = bin_op.a {
                                    Ok(())
                                } else {
                                    Err(format!(
                                        "cannot assign to {:?}",
                                        bin_op.a
                                    ).into())
                                }

                            } else if a_ty == b_ty {
                                let ty = a_ty;

                                let ty_set =
                                    if *ty == Type::Bool {
                                        TypeSet::Boolean
                                    } else if INTEGER_TYPES.contains(ty) {
                                        TypeSet::Integer
                                    } else if NUMERIC_TYPES.contains(ty) {
                                        TypeSet::Float
                                    } else {
                                        TypeSet::Other
                                    };

                                meta.ty = match (op_ty, ty_set) {
                                    (Arithmetic, TypeSet::Integer) |
                                    (Arithmetic, TypeSet::Float)
                                        => ty.clone(),
                                    (Equality, _) |
                                    (Comparison, TypeSet::Integer) |
                                    (Comparison, TypeSet::Float) |
                                    (Boolean, TypeSet::Boolean)
                                        => Type::Bool,
                                    (Range, TypeSet::Integer)
                                        // TODO fix
                                        => Type::Unknown,
                                    _ => {
                                        return Err(format!(
                                            "invalid type {:?} for operator {:?}",
                                            ty,
                                            bin_op.op
                                        ).into())
                                    }
                                };

                            } else {
                                return Err(format!(
                                    "type mismatch between \
                                    {:?} and {:?} for operator {:?}",
                                    a_ty,
                                    b_ty,
                                    bin_op.op
                                ).into())
                            }
                        }
                        Node::UnaryOp(_un_op, meta) => {
                            // TODO improve
                            meta.ty = Type::Bool;
                        }
                        Node::DefineVariable(ident, node, _) => {
                            let id = ident.as_id();
                            analysis.types.insert(*id, node.meta().ty.clone());
                        }
                        Node::Identifier(ident, meta) => {
                            let id = ident.as_id();
                            meta.ty = analysis.types[id].clone();
                        }
                        Node::Block(block, meta) => {
                            if !block.return_last {
                                meta.ty = Type::Null;
                            } else {
                                if block.nodes.len() == 0 {
                                    meta.ty = Type::Null;
                                } else {
                                    meta.ty = block.nodes[block.nodes.len()-1]
                                        .meta().ty.clone();
                                }
                            }
                        }
                        Node::ParenArgs(node, _, meta) => {
                            if let Node::Identifier(id, _) = &**node {
                                let id = id.as_id();
                                let ty = &analysis.types[id];
                                if let Type::Function(sig) = ty {
                                    meta.ty = if sig.1.len() != 0 {
                                        sig.1[0].clone()
                                    } else {
                                        Type::Null
                                    }
                                } else {
                                    return Err("expected function".into())
                                }
                            } else {
                                return Err("expected identifier".into())
                            }
                        }
                        _ => {}
                    }
                }

                Ok(())
            }
        )?;

        Ok(())
    }
}
