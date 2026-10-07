use std::{
    rc::Rc,
    any::Any,
    collections::HashMap,
};

// macro_rules! get_arg {
//     ($stack:ident, $type:ty) => {{
//         let ptr = $stack.current();
//         $stack.offset_add(std::mem::size_of::<$type>());
//         ptr as *mut $type
//     }};
//     ($stack:ident, offset $type:ty) => {{
//         let offset = $stack.current();
//         let ptr = unsafe {
//             $stack.get(*offset as usize)
//         };
//         $stack.offset_add(8);
//         ptr as *mut $type
//     }}
// }

#[allow(unused_macros)]
macro_rules! print_bytes {
    ($value:expr, $type:ty) => {{
        print!("binary representation of {} with len {}:\n    ", stringify!($type), std::mem::size_of::<$type>());
        for i in 0..std::mem::size_of::<$type>() {
            print!("{:0>2X} ", *(&$value as *const $type).cast::<u8>().add(i) as u8);
        }
        print!("\n\n");
    }}
}

#[macro_export]
macro_rules! op {
    ($self:ident, $type:expr, $op:tt) => {{
        match $type {
            Type::U8   => op!(calc $self, U8,   $op),
            Type::U16  => op!(calc $self, U16,  $op),
            Type::U32  => op!(calc $self, U32,  $op),
            Type::U64  => op!(calc $self, U64,  $op),
            Type::I8   => op!(calc $self, I8,   $op),
            Type::I16  => op!(calc $self, I16,  $op),
            Type::I32  => op!(calc $self, I32,  $op),
            Type::I64  => op!(calc $self, I64,  $op),
            Type::F32  => op!(calc $self, F32,  $op),
            Type::F64  => op!(calc $self, F64,  $op),
            Type::Bool => op!(calc $self, Bool, $op),
            _ => panic!()
        }
    }};

    // assume type Bool will never be used with non-binary operators
    (calc $_:ident, Bool, +)  => {{ panic!() }};
    (calc $_:ident, Bool, -)  => {{ panic!() }};
    (calc $_:ident, Bool, *)  => {{ panic!() }};
    (calc $_:ident, Bool, /)  => {{ panic!() }};
    (calc $_:ident, Bool, %)  => {{ panic!() }};

    // set $result to Bool for binary operators
    (calc $self:ident, $type:ident, ==) => {{ op!($self, $type, Bool, ==) }};

    // set $result to $type for everything else
    (calc $self:ident, $type:ident, $op:tt) => {{ op!($self, $type, $type, $op) }};

    ($self:ident, $type:ident, $result:ident, $op:tt) => {{
        unsafe {
            let b = $self.stack.pop().$type;
            let a = $self.stack.pop().$type;
            $self.stack.push(Value { $result: a $op b });
        }
    }};

    ($self:ident, $type:ident, $result:ident, $op:tt, $const:expr) => {{
        unsafe {
            let a = $self.stack.pop().$type;
            $self.stack.push(Value { $result: a $op $const });
        }
    }};

    // (calc $self:ident, $type:ident, +=) => {{ op!(calc_assign $self, $type, +=) }};

    // (calc_assign $self:ident, $type:ident, $op:tt) => {{
    //     unsafe {
    //         let b = $self.pop().0.$type;
    //         $self.peek_mut().0.$type $op b;
    //     }
    // }};
}

macro_rules! define_for_types {
    ($($value:ident: $type:ty,)*) => {
        #[allow(dead_code)]
        #[derive(Clone, Copy, Debug)]
        #[repr(u8)]
        enum Type {
             $( $value, )*
        }

        #[allow(non_snake_case)]
        #[derive(Clone, Copy)]
        union Value {
            $( $value: $type, )*
            Type: Type,
        }
    }
}

define_for_types! {
    Null: (),
    U8:  u8,
    U16: u16,
    U32: u32,
    U64: u64,
    I8:  i8,
    I16: i16,
    I32: i32,
    I64: i64,
    F32: f32,
    F64: f64,
    Bool: bool,
    Str: *mut String,
}

#[allow(dead_code)]
#[derive(Debug)]
enum Instruction {
    Debug,
    Exit,
    Call(Rc<Function>),
    CallInternal(fn(&mut Runtime)),
    Goto(usize),
    GotoConditional(usize),
    LoopUpdate(u64, u64, usize),
    Pop,
    Dup,
    DupFrom(usize),
    Swap(usize),
    MoveTo(usize),
    RotateLeft(usize),
    RotateRight(usize),
    PushNumber8(Type, u8),
    PushNumber16(Type, u16),
    PushNumber32(Type, u32),
    PushNumber64(Type, u64),
    PushString(*mut String),
    PushType(Type),
    Add(Type),
    Sub(Type),
    Mul(Type),
    Div(Type),
    Mod(Type),
    Equals(Type),
    And,
    Or,
}

use Instruction::*;

type Function = [Instruction];

// const INITIAL_STACK_SIZE: usize = 256;
const STACK_SIZE: usize = 8;

struct Stack {
    stack: Box<[Value; STACK_SIZE]>,
    next_stack_id: usize,
}

struct Runtime {
    stack: Stack,
    heap: HashMap<usize, Box<dyn Any>>,
    next_heap_id: usize,
}

impl Stack {
    fn new() -> Self {
        Self {
            // stack: Vec::with_capacity(INITIAL_STACK_SIZE),
            stack: Box::new([Value { Null: () }; STACK_SIZE]),
            next_stack_id: 0,
        }
    }

    fn push(&mut self, value: Value) {
        self.stack[self.next_stack_id] = value;
        self.next_stack_id += 1;
    }

    // fn replace(&mut self, ty: Type, value: Value) {
    //     let last_id = self.last_id();
    //     self.stack[last_id] = StackItem::new(value, ty);
    // }

    fn pop(&mut self) -> Value {
        self.next_stack_id = self.next_stack_id.saturating_sub(1);
        self.stack[self.next_stack_id]
    }

    fn peek(&self) -> &Value {
        &self.stack[self.last_id()]
    }

    fn peek_mut(&mut self) -> &mut Value {
        let last_id = self.last_id();
        &mut self.stack[last_id]
    }

    fn dup(&mut self, id: usize) {
        unsafe {
            std::ptr::copy(&self.stack[id], &mut self.stack[self.next_stack_id], 1);
            self.next_stack_id += 1;
        }
    }

    fn swap(&mut self, id: usize) {
        let last_id = self.last_id();
        self.stack.swap(id, last_id);
    }

    fn move_to(&mut self, id: usize) {
        self.next_stack_id -= 1;
        self.stack[id] = self.stack[self.next_stack_id];
    }

    fn len(&self) -> usize {
        self.next_stack_id
    }

    fn last_id(&self) -> usize {
        self.next_stack_id-1
    }
}

impl Runtime {
    fn new() -> Self {
        Self {
            stack: Stack::new(),
            heap: HashMap::new(),
            next_heap_id: 0,
        }
    }

    fn heap_add(&mut self, b: Box<dyn Any>) -> *mut u8 {
        let id = self.next_heap_id;
        self.heap.insert(id, b);
        self.next_heap_id += 1;
        Box::as_mut_ptr(&mut self.heap.get_mut(&id).unwrap()) as *mut u8
    }

    fn call(&mut self, func: Rc<Function>) {
        let mut i = 0;
        while i < (*func).len() {
            let instruction = &func[i];
            // eprintln!("exec instr {i}: {instruction:?}");
            i += 1;
            match instruction {
                Debug => debug_hexdump(self),
                Exit => { break }
                Call(func2) => self.call(func2.clone()),
                CallInternal(internal_func) => internal_func(self),
                Pop => { self.stack.pop(); }
                Dup => self.stack.dup(self.stack.last_id()),
                Swap(id) => self.stack.swap(*id),
                DupFrom(id) => self.stack.dup(*id),
                MoveTo(id) => self.stack.move_to(*id),
                RotateLeft(count) => {
                    let len = self.stack.len();
                    self.stack.stack[len-count..len].rotate_left(1);
                }
                RotateRight(count) => {
                    let len = self.stack.len();
                    self.stack.stack[len-count..len].rotate_right(1);
                }
                PushNumber8(ty, u_8) => {
                    match ty {
                        Type::U8 => self.stack.push(Value { U8: *u_8 }),
                        Type::I8 => self.stack.push(Value { I8: u8::cast_signed(*u_8) }),
                        _ => panic!()
                    }
                }
                PushNumber16(ty, u_16) => {
                    match ty {
                        Type::U16 => self.stack.push(Value { U16: *u_16 }),
                        Type::I16 => self.stack.push(Value { I16: u16::cast_signed(*u_16) }),
                        _ => panic!()
                    }
                }
                PushNumber32(ty, u_32) => {
                    match ty {
                        Type::U32 => self.stack.push(Value { U32: *u_32 }),
                        Type::I32 => self.stack.push(Value { I32: u32::cast_signed(*u_32) }),
                        Type::F32 => self.stack.push(Value { F32: f32::from_bits(*u_32) }),
                        _ => panic!()
                    }
                }
                PushNumber64(ty, u_64) => {
                    match ty {
                        Type::U64 => self.stack.push(Value { U64: *u_64 }),
                        Type::I64 => self.stack.push(Value { I64: u64::cast_signed(*u_64) }),
                        Type::F64 => self.stack.push(Value { F64: f64::from_bits(*u_64) }),
                        _ => panic!()
                    }
                }
                PushString(string) => self.stack.push(Value { Str: string.clone() }),
                PushType(ty) => self.stack.push(Value { Type: *ty }),
                Goto(index) => { i = *index }
                GotoConditional(index) => {
                    unsafe {
                        if self.stack.pop().Bool == false {
                            i = *index;
                        }
                    }
                }
                LoopUpdate(max, step, jump_index) => {
                    unsafe {
                        self.stack.peek_mut().U64 += step;
                        if self.stack.peek().U64 != *max {
                            i = *jump_index;
                        }
                    }
                }
                Add(ty) => op!(self, ty, +),
                Sub(ty) => op!(self, ty, -),
                Mul(ty) => op!(self, ty, *),
                Div(ty) => op!(self, ty, /),
                Mod(ty) => op!(self, ty, %),
                Equals(ty) => op!(self, ty, ==),
                And => op!(self, Bool, Bool, &&),
                Or => op!(self, Bool, Bool, ||),
            }
        }
    }
}

#[allow(unused)]
fn debug_hexdump(runtime: &mut Runtime) {
    println!("current stack:");
    for (i, value) in runtime.stack.stack[..runtime.stack.next_stack_id].iter().enumerate() {
        print!("    {i}: ");
        for i in 0..std::mem::size_of::<Value>() {
            unsafe {
                print!("{:0>2X} ", *(value as *const Value).cast::<u8>().add(i) as u8);
            }
        }
        println!();
    }
}

fn internal_tostring(runtime: &mut Runtime) {
    unsafe {
        let ty = runtime.stack.pop().Type;
        let num = runtime.stack.peek();
        let string = match ty {
            Type::U16 => num.U16.to_string(),
            Type::U32 => num.U32.to_string(),
            Type::U64 => num.U64.to_string(),
            _ => todo!()
        };
        let b = Box::new(string);
        let ptr = runtime.heap_add(b);
        // let ptr = (&mut runtime.stack.heap[id]) as *mut String;
        runtime.stack.push(Value { Str: ptr as *mut String });
    }
}

fn internal_print(runtime: &mut Runtime) {
    unsafe {
        let string = runtime.stack.peek().Str;
        // print_bytes!(*string, String);
        println!("{}", *string);
    }
}

#[allow(unused)]
fn internal_str_append(runtime: &mut Runtime) {
    unsafe {
        let src = runtime.stack.pop().Str as *mut String;
        let rhs = runtime.stack.pop().Str as *mut String;
        // print_bytes!(*rhs, String);
        // print_bytes!(*src, String);
        (*src).push_str(&*rhs);
    }
}

#[test]
fn simple() {
    println!();
    let mut runtime = Runtime::new();
    runtime.call(Rc::new([
        PushNumber16(Type::U16, 123),
        PushType(Type::U16),
        CallInternal(internal_tostring),
        RotateLeft(2),
        Pop,
        PushString(&mut "hejsan".to_string()),
        Debug,
        CallInternal(internal_print),
        Pop,
        CallInternal(internal_print),
        Pop,
    ]));
    assert_eq!(runtime.stack.len(), 0);
}

#[test]
fn exclaim_513() {
    println!();
    let mut runtime = Runtime::new();
    runtime.call(Rc::new([
        PushString(&mut "!!!".to_string()),
        PushNumber16(Type::U16, 256),
        PushNumber16(Type::U16, 257),
        Add(Type::U16),
        PushType(Type::U16),
        CallInternal(internal_tostring),
        DupFrom(0),
        DupFrom(2),
        CallInternal(internal_str_append),
        CallInternal(internal_print),
        Debug,
    ]));
}

#[allow(unused)]
fn test_fib() {
    println!();
    let mut runtime = Runtime::new();
    runtime.call(Rc::new([
        PushNumber64(Type::U64, 0),
        PushNumber64(Type::U64, 1),
        PushNumber64(Type::U64, 0),
        DupFrom(0),
        DupFrom(1),
        Add(Type::U64),
        Swap(1),
        Swap(0),
        Pop,
        LoopUpdate(92, 1, 3),
        Pop,
        PushType(Type::U64),
        CallInternal(internal_tostring),
        CallInternal(internal_print),
    ]));
}

#[allow(unused)]
fn test_fib_mod() {
    let mut runtime = Runtime::new();
    runtime.call(Rc::new([
        PushNumber64(Type::U64, 0),
        PushNumber64(Type::U64, 1),
        PushNumber64(Type::U64, 0),
        DupFrom(0),
        DupFrom(1),
        Add(Type::U64),
        PushNumber64(Type::U64, 1_000_000),
        Mod(Type::U64),
        Swap(1),
        Swap(0),
        Pop,
        LoopUpdate(4_000_000, 1, 3),
        Pop,
        PushType(Type::U64),
        CallInternal(internal_tostring),
        CallInternal(internal_print),
    ]));
}

#[test]
fn fib() {
    test_fib();
}

#[test]
fn mod_fib() {
    test_fib_mod();
}
