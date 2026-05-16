pub mod buffer;
pub mod token;
pub mod tokenizer;
pub mod node;
pub mod parser;
pub mod syntax;
pub mod stdlib;
pub mod runtime;

use {
    buffer::Buffer,
    tokenizer::Tokenizer,
    parser::Parser,
    runtime::*,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn setup() -> (Runtime, Scope) {
    let mut runtime = Runtime::new();
    let scope = Scope::new(None);
    stdlib::init(&mut runtime.globals);
    (runtime, scope)
}

pub fn parse(string: String) -> Result<node::Block> {
    let debug = if let Ok(debug) = std::env::var("SEMMEL_DEBUG")
        { debug == "1" } else { false };

    let buffer = Buffer::new(string.chars().collect());
    let mut tokenizer = Tokenizer::new(buffer);
    let (tokens, metas) = tokenizer.tokenize()?;

    if debug {
        println!("tokens:");
        for (token, meta) in tokens.iter().zip(&metas) {
            let pos = format!("{}:{}", meta.row+1, meta.col+1);
            println!("    {pos:5}  {token:?}");
        }
    }

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

    if debug {
        println!("\nnodes:");
        for node in block.nodes.iter() {
            println!("{node:#?},");
        }
        println!();
    }

    Ok(block)
}
