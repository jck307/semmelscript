mod buffer;
mod token;
mod tokenizer;
mod node;
mod parser;
mod syntax;
mod runtime;
mod stdlib;

use {
    buffer::Buffer,
    tokenizer::Tokenizer,
    parser::Parser,
    runtime::*,
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn parse(string: String) -> Result<node::Block> {
    let buffer = Buffer::new(string.chars().collect());
    let mut tokenizer = Tokenizer::new(buffer);
    let (tokens, metas) = tokenizer.tokenize()?;

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

    Ok(block)
}

fn run() -> Result<()> {
    let [_, path]: [String; 2] = std::env::args()
        .collect::<Vec<_>>().try_into()
        .unwrap_or_else(|_| panic!("Expected 1 argument!"));

    let string = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Could not read file {path}: {e}"));

    let block = parse(string)?;

    // println!("tokens:");
    // for (token, meta) in tokens.iter().zip(&metas) {
    //     let pos = format!("{}:{}", meta.row+1, meta.col+1);
    //     println!("    {pos:5}  {token:?}");
    // }

    // println!("nodes:");
    // for node in block.nodes.iter() {
    //     println!("    {node:#?}");
    // }

    let mut runtime = Runtime::new();
    let mut scope = Scope::new(None);
    stdlib::init(&mut runtime.globals);
    block.eval(&mut runtime, &mut scope)?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
    }
}
