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
                    Node::ParenArgs(node, args, meta) => {
                        let Node::Identifier(id, _) = &**node else {
                            return Err("expected identifier".into())
                        };
                        let id = id.as_id();
                        let ty = &analysis.types[id];
                        let Type::Function(sig) = ty else {
                            return Err("expected function".into())
                        };
                        if sig.0.len() != args.len() {
                            return Err(format!(
                                "expected {} arguments, got {}",
                                sig.0.len(),
                                args.len()
                            ).into())
                        }
                        for i in 0..(args.len()) {
                            let exp_ty = &sig.0[i];
                            let giv_ty = &args[i].meta().ty;
                            if *exp_ty != Type::Any && exp_ty != giv_ty {
                                return Err(format!(
                                    "expected type {:?} for argument {}, got {:?}",
                                    exp_ty,
                                    i + 1,
                                    giv_ty
                                ).into())
                            }
                        }
                        meta.ty = if sig.1.len() != 0 {
                            sig.1[0].clone()
                        } else {
                            Type::Null
                        };
                    }
                    _ => {}
                }
            }

            Ok(())
        }
    )?;

    Ok(())
}
