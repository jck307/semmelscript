#[derive(Clone, Default, Debug, PartialEq)]
#[allow(dead_code)]
pub struct FunctionSignature(pub Vec<Type>, pub Vec<Type>);

impl FunctionSignature {
    pub fn num_args(&self) -> usize {
        self.0.len()
    }

    pub fn num_return_values(&self) -> usize {
        self.1.len()
    }
}

macro_rules! define_types {
    ($($value:ident: $type:ty,)*) => {
        #[allow(dead_code)]
        #[derive(Clone, Debug, PartialEq)]
        #[repr(u16)]
        pub enum Type {
            $( $value, )*
            Unknown,
            Function(Box<FunctionSignature>),
        }

        #[allow(non_snake_case)]
        #[derive(Clone, Copy)]
        pub union Value {
            $( pub $value: $type, )*
            pub Type: *const Type,
        }
    }
}

define_types! {
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
    String: *mut String,
    // Ptr: *mut u8,
}

pub static NUMERIC_TYPES: &[Type] = &[
    Type::U8,
    Type::U16,
    Type::U32,
    Type::U64,
    Type::I8,
    Type::I16,
    Type::I32,
    Type::I64,
    Type::F32,
    Type::F64,
];
