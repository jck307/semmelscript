macro_rules! define_types {
    ($($value:ident: $type:ty,)*) => {
        #[allow(dead_code)]
        #[derive(Clone, Copy, Debug)]
        #[repr(u16)]
        pub enum Type {
            $( $value, )*
        }

        #[allow(non_snake_case)]
        #[derive(Clone, Copy)]
        pub union Value {
            $( pub $value: $type, )*
            pub Type: Type,
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
    Str: *mut String,
    // Ptr: *mut u8,
}
