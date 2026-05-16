use crate::*;
use crate::node::*;
use std::collections::HashMap;

type IntegerType = i32;

// TODO replace some 'name' with 'ident'

use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum RuntimeError {
        ExpectedType(typ: Type) {}
        ExpectedArgs(len: usize) {}
        NameError(name: Box<str>) {}
    }
}

use RuntimeError::*;

#[macro_export]
macro_rules! expect_type {
    ($value:expr, $type:ident) => {{
        use crate::runtime::{Object, Type, RuntimeError};
        match $value {
            Object::$type(value) => value,
            _ => { return Err( RuntimeError::ExpectedType(Type::$type).into()); }
        }}
    }
}

pub struct Runtime {
    pub globals: Scope,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub objects: Vec<Object>,
    pub names: HashMap<Box<str>, usize>,
    pub parent: Option<*mut Scope>,
}

#[derive(Debug, Clone)]
pub enum Function {
    Pointer(fn(&mut Runtime, &mut Scope) -> Result<Object>),
    Block(Block),
}

#[derive(Debug)]
pub enum Type {
    // Null,
    String,
    Integer,
    Boolean,
    Function,
    List,
}

#[derive(Debug, Clone)]
pub enum Object {
    Null,
    String(String),
    Integer(IntegerType),
    Boolean(bool),
    Function {
        func: Box<Function>,
        args: Vec<Box<str>>,
    },
    List(Vec<Object>),
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            globals: Scope::new(None),
        }
    }
}

impl Scope {
    pub fn new(parent: Option<*mut Scope>) -> Self {
        Self {
            names: HashMap::new(),
            objects: Vec::new(),
            parent,
        }
    }

    fn add_object(&mut self, object: Object) -> usize {
        self.objects.push(object);
        self.objects.len() - 1
    }

    // TODO remove runtime?
    pub fn define(&mut self, name: &str, object: Object) {
        assert!(!self.names.contains_key(name)); // TODO fix
        let id = self.add_object(object);
        self.names.insert(name.into(), id);
    }

    pub fn update(&mut self, runtime: &mut Runtime, name: &str, object: Object) -> Result<()> {
        if let Some(id) = self.names.get(name) {
            self.objects.insert(*id, object);
            Ok(())

        } else if let Some(parent) = self.parent {
            unsafe {
                (*parent).update(runtime, name, object)
            }

        } else {
            Err(NameError(name.into()).into())
        }
    }

    pub fn get(&mut self, runtime: &Runtime, name: &str) -> Result<Object> {
        if let Some(id) = self.names.get(name) {
            Ok(self.objects[*id].clone())
        } else {
            if let Some(parent) = self.parent {
                unsafe {
                    (*parent).get(runtime, name)
                }
            } else {
                match runtime.globals.names.get(name) {
                    Some(id) => {
                        Ok(runtime.globals.objects[*id].clone())
                    }
                    None => Err(NameError(name.into()).into())
                }
            }
        }
    }

    fn root(&mut self) -> *mut Self {
        if let Some(parent) = self.parent {
            unsafe {
                (*parent).root()
            }
        } else {
            self
        }
    }
}

pub trait Evaluate {
    // evaluates the value of a node
    fn eval(&self, _runtime: &mut Runtime, _scope: &mut Scope) -> Result<Object> {
        // TODO remove
        unimplemented!()
    }
}

impl Evaluate for Node {
    fn eval(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
        match self {
            Self::ParenArgs(root, args) => {
                match root.eval(runtime, scope)? {
                    Object::Function { func, args: arg_names } => {
                        if args.len() != arg_names.len() {
                            return Err(ExpectedArgs(arg_names.len()).into());
                        }

                        let mut func_scope = Scope::new(Some(scope.root()));
                        for (i, arg_name) in arg_names.iter().enumerate() {
                            let arg_node: &Node = &args[i];
                            let arg = arg_node.eval(runtime, scope)?;
                            func_scope.define(arg_name, arg);
                        }

                        match *func {
                            Function::Pointer(ptr) => {
                                ptr(runtime, &mut func_scope)
                            }
                            Function::Block(block) => {
                                block.eval(runtime, &mut func_scope)
                            }
                        }
                    }
                    _ => Err(ExpectedType(Type::Function).into())
                }
            }

            Self::Statement(node) => node.eval(runtime, scope), 
            Self::BinaryOp(node) => node.eval(runtime, scope), 

            Self::Block(node) => {
                node.eval(runtime, &mut scope.clone())
            }

            Self::Identifier(ident) => scope.get(runtime, ident),
            Self::String(string) => Ok(Object::String(string.to_string())),
            Self::Integer(integer) => Ok(Object::Integer(*integer)),
            Self::Boolean(boolean) => Ok(Object::Boolean(*boolean)),
            Self::List(list) => {
                let result: Result<Vec<Object>> = list.iter()
                    .map(|n| n.eval(runtime, scope)).collect();
                Ok(Object::List(result?))
            }
        }
    }
}

impl Evaluate for Block {
    fn eval(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
        for node in &self.nodes {
            let _ = node.eval(runtime, scope)?;
        }

        Ok(Object::Null)
    }
}

impl Evaluate for Statement {
    fn eval(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
        match self {
            Self::DefineVariable(name, value) => {
                let value = value.eval(runtime, scope)?;
                scope.define(name, value);
                Ok(Object::Null)
            }
            Self::DefineFunction(name, args, block) => {
                // TODO replace cloning with pointer or something?
                scope.define(name, Object::Function {
                    func: Box::new(Function::Block(block.clone())),
                    args: args.clone(),
                });
                Ok(Object::Null)
            }
            Self::If(condition, block, ext) => {
                if expect_type!(condition.eval(runtime, scope)?, Boolean) {
                    block.eval(runtime, scope)?;
                    if let Some(ext) = ext {
                        match &**ext {
                            // else statements:
                            Node::Block(ext_block) => {
                                ext_block.eval(runtime, scope)?;
                            }
                            // elif statements:
                            Node::Statement(ext_statement) => {
                                match ext_statement {
                                    Statement::If(..) => {
                                        ext_statement.eval(runtime, scope)?;
                                    }
                                    _ => unreachable!()
                                }
                            }
                            _ => unreachable!()
                        };
                    }
                }

                Ok(Object::Null)
            }
            Self::For(ident, sequence, block) => {
                let sequence = expect_type!(sequence.eval(runtime, scope)?, List);
                for object in sequence.iter() {
                    // TODO reuse scope instead
                    let mut scope = Scope::new(Some(scope));
                    scope.define(ident, object.clone());
                    block.eval(runtime, &mut scope)?;
                }
                Ok(Object::Null)
            }
        }
    }
}

impl Evaluate for BinaryOp {
    fn eval(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
        use crate::token::Operator::*;

        Ok(match self.op {
            Add | Sub | Mul | Div | Pow | Mod |
            Equal | Inequal | Less | LessEqual | Greater | GreaterEqual => {
                let a = self.a.eval(runtime, scope)?;
                let b = self.b.eval(runtime, scope)?;

                match self.op {
                    Add => {
                        // string concatenation
                        match a {
                            Object::String(a) => {
                                let b = expect_type!(b, String);
                                return Ok(Object::String(a + &b))
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

                let a = expect_type!(a, Integer);
                let b = expect_type!(b, Integer);

                match self.op {
                    Add | Sub | Mul | Div | Pow | Mod => {
                        Object::Integer(match self.op {
                            Add => a + b,
                            Sub => a - b,
                            Mul => a * b,
                            Div => a / b,
                            Pow => a.pow(b.try_into()
                                .expect("expected exponent of type u32")),
                            Mod => a % b,
                            _ => unreachable!()
                        })
                    }
                    Equal | Inequal | Less | LessEqual | Greater | GreaterEqual => {
                        Object::Boolean(match self.op {
                            Equal => a == b,
                            Inequal => a != b,
                            Less => a < b,
                            LessEqual => a <= b,
                            Greater => a >= b,
                            GreaterEqual => a >= b,
                            _ => unreachable!()
                        })
                    }
                    _ => unreachable!()
                }
            }

            And | Or => {
                let a = expect_type!(self.a.eval(runtime, scope)?, Boolean);
                let b = expect_type!(self.b.eval(runtime, scope)?, Boolean);

                Object::Boolean(match self.op {
                    And => a && b,
                    Or => a || b,
                    _ => unreachable!()
                })
            }

            RangeExcl => {
                let a = expect_type!(self.a.eval(runtime, scope)?, Integer);
                let b = expect_type!(self.b.eval(runtime, scope)?, Integer);
                Object::List((a..b).map(|i| Object::Integer(i)).collect())
            }

            SetValue => {
                let Node::Identifier(ref name) = self.a else { unreachable!() };
                let value = self.b.eval(runtime, scope)?;
                scope.update(runtime, &name, value)?;
                Object::Null
            }
            
            _ => unreachable!()
        })
    }
}
