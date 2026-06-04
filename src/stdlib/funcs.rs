use super::*;
use std::process::Command;

pub fn println(_runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
    println!("{}", get!(scope, text, String));
    Ok(Object::Null)
}

pub fn print(_runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
    print!("{}", get!(scope, text, String));
    Ok(Object::Null)
}

pub fn source(runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
    let path = get!(scope, path, String).clone();
    let scope: &mut Scope = unsafe { &mut *scope.parent.unwrap() };
    let string = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Could not read file {path}: {e}"));
    let block = crate::parse(string)?;
    block.eval(runtime, scope)?;
    Ok(Object::Null)
}

pub fn tostring(_runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
    // TODO use the same formatting as parser::node::Node
    let obj = scope.get("value")?;
    Ok(Object::String(match obj {
        Object::String(string) => string.to_string(),
        Object::Integer(integer) => integer.to_string(),
        Object::Float(float) => float.to_string(),
        Object::List(list) => unsafe { format!("{:?}", *(*list).vec) }
        _ => format!("{obj:?}")
    }))
}

pub fn call(_runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
    let (shell, flag) = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

    let stdout = Command::new(shell).arg(flag)
        .arg(get!(scope, cmd, String))
        .output()
        .expect("command failed")
        .stdout; // TODO fix

    let mut stdout: String = stdout.iter().map(|b| *b as char).collect();
    
    // remove trailing newline
    if let Some(ch) = stdout.bytes().last() {
        if ch == b'\n' {
            stdout.remove(stdout.len() - 1);
        }
    }

    Ok(Object::String(stdout))
}

pub fn get(_runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
    let index: usize = get!(scope, index, Integer).clone().try_into().unwrap();
    unsafe {
        let arr = &(*get!(scope, arr, List)).vec;
        Ok(arr[index].clone())
    }
}

pub fn set(_runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
    let index: usize = get!(scope, index, Integer).clone().try_into().unwrap();
    let value = scope.get("value")?.clone();
    unsafe {
        (&mut (*get!(scope, arr, List)).vec)[index] = value;
    }
    Ok(Object::Null)
}

pub fn drop(runtime: &mut Runtime, scope: &mut Scope) -> Result<Object> {
    use std::mem::ManuallyDrop as ManDrop;
    let heap_obj: *const HeapObject = match scope.get("var")? {
        Object::List(obj) => unsafe {
            ManDrop::drop(&mut (*obj).vec);
            obj
        }
        _ => { return Ok(Object::Null) }
    };
    for (i, obj) in runtime.heap.iter().rev().enumerate() {
        if heap_obj == &raw const *obj {
            runtime.heap.remove(i);
            break
        }
    }
    Ok(Object::Null)
}
