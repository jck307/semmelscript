use crate::*;

pub fn tostring(runtime: &mut Runtime) {
    unsafe {
        let ty = Type::I32; // runtime.stack.pop().Type;
        let num = runtime.stack.pop();
        let string = match ty {
            Type::U16 => num.U16.to_string(),
            Type::U32 => num.U32.to_string(),
            Type::I32 => num.I32.to_string(),
            Type::U64 => num.U64.to_string(),
            _ => todo!()
        };
        let b = Box::new(string);
        let ptr = runtime.heap_add(b);
        // let ptr = (&mut runtime.stack.heap[id]) as *mut String;
        runtime.stack.push(Value { Str: ptr as *mut String });
    }
}

pub fn println(runtime: &mut Runtime) {
    unsafe {
        let string = runtime.stack.pop().Str;
        // print_bytes!(*string, String);
        println!("{}", *string);
    }
}

pub fn str_append(runtime: &mut Runtime) {
    unsafe {
        let src = runtime.stack.pop().Str as *mut String;
        let rhs = runtime.stack.pop().Str as *mut String;
        // print_bytes!(*rhs, String);
        // print_bytes!(*src, String);
        (*src).push_str(&*rhs);
    }
}
