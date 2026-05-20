use crate::*;
use crate::node::*;
use std::collections::HashMap;

// TODO replace some 'name' with 'ident'

use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum RuntimeError {
        ExpectedType(typ: Type, found: Type) {}
        ExpectedArgs(len: usize) {}
        ExpectedNumber(found: Type) {}
        NameError(name: Box<str>) {}
    }
}

use RuntimeError::*;

#[macro_export]
macro_rules! expect_type {
    ($value:expr, $type:ident) => {{
        use crate::runtime::{Object, Type, RuntimeError};
        let value = $value;
        match value {
            Object::$type(value) => value,
            _ => { return Err( RuntimeError::ExpectedType(Type::$type, value.get_type()).into()); }
        }}
    }
}

pub struct Runtime {
    pub globals: Scope,
    objects: HashMap<Pointer, Object>,
    next_id: Pointer,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub(crate) parent: Option<*mut Scope>,
    runtime: *mut Runtime,
    names: HashMap<Box<str>, Pointer>,
}

unsafe impl Send for Scope {}

#[derive(Debug, Clone)]
pub enum Function {
    Pointer(fn(&mut Runtime, &mut Scope) -> Result<Object>),
    Block(Block),
}

#[derive(Debug)]
pub enum Type {
    Null,
    Pointer,
    String,
    Integer,
    Float,
    Boolean,
    Function,
    List,
}

#[derive(Debug, Clone)]
pub enum Object {
    Null,
    Pointer(Pointer),
    String(String),
    Integer(Integer),
    Float(Float),
    Boolean(bool),
    Function {
        func: Box<Function>,
        args: Vec<Box<str>>,
    },
    List(Vec<Object>),
}

impl Object {
    pub fn get_type(&self) -> Type {
        use Type::*;
        match self {
            Self::Null => Null,
            Self::Pointer(_) => Pointer,
            Self::String(_) => String,
            Self::Integer(_) => Integer,
            Self::Float(_) => Float,
            Self::Boolean(_) => Boolean,
            Self::Function { .. } => Function,
            Self::List(_) => List,
        }
    }
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            globals: Scope::new(std::ptr::null_mut(), None),
            objects: HashMap::new(),
            next_id: 0,
        }
    }
    
    pub fn next_id(&mut self) -> Pointer {
        self.next_id += 1;
        self.next_id - 1
    }
}

pub fn set_runtime_pointer(runtime: &mut Runtime, scope: &mut Scope) {
    runtime.globals.runtime = runtime;
    scope.runtime = runtime;
}

impl Scope {
    pub fn new(runtime: *mut Runtime, parent: Option<*mut Scope>) -> Self {
        Self {
            runtime,
            parent,
            names: HashMap::new(),
        }
    }

    fn add_object(&mut self, object: Object) -> Pointer {
        unsafe {
            let id = (*self.runtime).next_id();
            (*self.runtime).objects.insert(id, object);
            id
        }
    }

    pub fn define(&mut self, name: &str, object: Object) {
        assert!(!self.names.contains_key(name)); // TODO fix
        let id = self.add_object(object);
        self.names.insert(name.into(), id);
    }

    pub fn update(&mut self, name: &str, object: Object) -> Result<()> {
        if let Some(id) = self.names.get(name) {
            unsafe {
                (*self.runtime).objects.insert(*id, object);
            }
            Ok(())

        } else if let Some(parent) = self.parent {
            unsafe {
                (*parent).update(name, object)
            }

        } else {
            Err(NameError(name.into()).into())
        }
    }

    pub fn get(&mut self, name: &str) -> Result<Object> {
        if let Some(id) = self.names.get(name) {
            unsafe {
                Ok((&(*self.runtime).objects)[id].clone())
            }
        } else {
            if let Some(parent) = self.parent {
                unsafe {
                    (*parent).get(name)
                }
            } else {
                unsafe {
                    if !(*self.runtime).globals.runtime.is_null() {
                        if let Some(id) = (*self.runtime).globals.names.get(name) {
                            return Ok((&(*self.runtime).objects)[id].clone())
                        }
                    }
                }
                Err(NameError(name.into()).into())
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

pub fn call_function(runtime: &mut Runtime, scope: &mut Scope, object: Object, mut args: Vec<Object>) -> Result<Object> {
    match object {
        Object::Function { func, args: arg_names } => {
            if args.len() != arg_names.len() {
                return Err(ExpectedArgs(arg_names.len()).into());
            }

            let mut func_scope = Scope::new(runtime, Some(scope.root()));
            for arg_name in arg_names.iter() {
                func_scope.define(arg_name, args.remove(0));
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
        _ => Err(ExpectedType(Type::Function, object.get_type()).into())
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
            Self::ParenArgs(root, arg_nodes) => {
                let object = root.eval(runtime, scope)?;
                let mut args = Vec::new();
                for arg in arg_nodes.iter() {
                    args.push(arg.eval(runtime, scope)?);
                }
                call_function(runtime, scope, object, args)
            }

            Self::Statement(node) => node.eval(runtime, scope), 
            Self::BinaryOp(node) => node.eval(runtime, scope), 

            Self::Block(node) => {
                node.eval(runtime, &mut scope.clone())
            }

            Self::Identifier(ident) => scope.get(ident),
            Self::String(string) => Ok(Object::String(string.to_string())),
            Self::Integer(integer) => Ok(Object::Integer(*integer)),
            Self::Float(float) => Ok(Object::Float(*float)),
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
        let mut return_value = Object::Null;

        for node in &self.nodes {
            return_value = node.eval(runtime, scope)?;
        }

        Ok(return_value)
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
                    let mut scope = Scope::new(runtime, Some(scope));
                    scope.define(ident, object.clone());
                    block.eval(runtime, &mut scope)?;
                }
                Ok(Object::Null)
            }
        }
    }
}

macro_rules! calculate {
    ($self:ident, $type:ident, $a:ident, $b:ident, $pow:ident) => {{
        match $self.op {
            Add | Sub | Mul | Div | Pow | Mod => {
                Object::$type(match $self.op {
                    Add => $a + $b,
                    Sub => $a - $b,
                    Mul => $a * $b,
                    Div => $a / $b,
                    Pow => $a.$pow($b.try_into()
                        .expect("invalid exponent type")),
                    Mod => $a % $b,
                    _ => unreachable!()
                })
            }
            Equal | Inequal | Less | LessEqual | Greater | GreaterEqual => {
                Object::Boolean(match $self.op {
                    Equal => $a == $b,
                    Inequal => $a != $b,
                    Less => $a < $b,
                    LessEqual => $a <= $b,
                    Greater => $a >= $b,
                    GreaterEqual => $a >= $b,
                    _ => unreachable!()
                })
            }
            _ => unreachable!()
        }
    }}
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

                match a {
                    Object::Integer(a) => {
                        match b {
                            Object::Integer(b) => {
                                return Ok(calculate!(self, Integer, a, b, pow))
                            }
                            Object::Float(b) => {
                                let a = a as f32;
                                return Ok(calculate!(self, Float, a, b, powf))
                            }
                            _ => {}
                        }
                    }
                    Object::Float(a) => {
                        match b {
                            Object::Integer(b) => {
                                let b = b as f32;
                                return Ok(calculate!(self, Float, a, b, powf))
                            }
                            Object::Float(b) => {
                                return Ok(calculate!(self, Float, a, b, powf))
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

                return Err(ExpectedNumber(a.get_type()).into())
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

            Assign => {
                let Node::Identifier(ref name) = self.a else { unreachable!() };
                let value = self.b.eval(runtime, scope)?;
                scope.update(&name, value)?;
                Object::Null
            }

            AddAssign | SubAssign | MulAssign | DivAssign | PowAssign | ModAssign => todo!(),
            _ => unreachable!()
        })
    }
}
