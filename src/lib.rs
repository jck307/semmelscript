mod buffer;
mod token;
mod tokenizer;
mod node;
mod parser;
mod syntax;
mod stdlib;
mod runtime;
mod codegen;
mod types;
mod instruction;

use std::{
    rc::Rc,
};
use {
    buffer::Buffer,
    tokenizer::Tokenizer,
    parser::Parser,
    instruction::Instruction,
    runtime::Runtime,
    types::{Type, Value},
    codegen::{CodeGenerator, Scope, Generate},
};

type Result<T> = std::result::Result<T,
    Box<dyn std::error::Error + Send + Sync + 'static>>;
type Integer = i64;
type Float = f64;

fn parse(string: String) -> Result<node::Block> {
    // let debug = if let Ok(debug) = std::env::var("SEMMEL_DEBUG")
    //     { debug == "1" } else { false };
     
    let buffer = Buffer::new(string.chars().collect());
    let mut tokenizer = Tokenizer::new(buffer);
    let (tokens, metas) = tokenizer.tokenize()?;

    // if debug {
    //     println!("tokens:");
    //     for (token, meta) in tokens.iter().zip(&metas) {
    //         let pos = format!("{}:{}", meta.row+1, meta.col+1);
    //         println!("    {pos:5}  {token:?}");
    //     }
    // }

    let tokens = Buffer::new(tokens.into());
    let mut parser = Parser::new(tokens);

    let block = match parser.parse() {
        Ok(node::Node::Block(block)) => block,
        Err(err) => {
            let meta = &metas.get(parser.tokens.i).unwrap_or_else(||
                &metas[metas.len()-1]);
            return Err(format!("syntax error at {}:{}: {err}",
                meta.row+1, meta.col+1).into())
        }
        _ => unreachable!()
    };

    // if debug {
    //     for node in block.nodes.iter() {
    //         println!("{node:#?},");
    //     }
    // }

    Ok(block)
}

pub fn run() -> Result<()> {
    let debug = if let Ok(debug) = std::env::var("SEMMEL_DEBUG")
        { debug == "1" } else { false };

    let [_, path]: [String; 2] = std::env::args()
        .collect::<Vec<_>>().try_into()
        .unwrap_or_else(|_| panic!("Expected 1 argument!"));

    let string = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Could not read file {path}: {e}"));

    let block = parse(string)?;
    let mut generator = CodeGenerator::new();
    generator.builtins = stdlib::get_builtins();
    let mut scope = Scope::default();
    block.generate(&mut generator, &mut scope)?;
    generator.do_final()?;

    if debug {
        for (i, func) in generator.functions_final.iter().enumerate() {
            println!("function {i}:");
            for i in 0..func.len() {
                println!("    {:?}", func[i]);
            }
        }
    }

    let mut runtime = Runtime::new();
    runtime.call(generator.functions_final[0].clone().into());

    if debug {
        runtime.call(Rc::new([Instruction::Debug]));
    }

    // let (mut runtime, mut scope) = setup!();
    // block.eval(&mut runtime, &mut scope)?;

    Ok(())
}
