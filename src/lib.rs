mod buffer;
mod token;
mod tokenizer;
mod node;
mod parser;
mod syntax;
mod stdlib;
mod runtime;
mod types;
mod instruction;
mod traverse;
mod analysis;
mod codegen;
mod debug;

pub use lib::*;

mod lib {
    use {
        std::collections::HashMap,
        super::{
            *,
            buffer::Buffer,
            tokenizer::Tokenizer,
            parser::Parser,
            runtime::Runtime,
            analysis::Analysis,
            node::Node,
            types::Type,
            codegen::{CodeGenerator, Generate},
        },
    };

    pub type Result<T> = std::result::Result<T,
        Box<dyn std::error::Error + Send + Sync + 'static>>;
    pub type Integer = i64;
    pub type Float = f64;

    fn parse(string: String) -> Result<Node> {
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
            Ok(block) => block,
            Err(err) => {
                let meta = &metas.get(parser.tokens.i).unwrap_or_else(||
                    &metas[metas.len()-1]);
                return Err(format!("syntax error at {}:{}: {err}",
                    meta.row+1, meta.col+1).into())
            }
        };

        // if debug {
        //     for node in block.nodes.iter() {
        //         println!("{node:#?},");
        //     }
        // }

        Ok(block)
    }

    pub fn run() -> Result<()> {
        let [_, path]: [String; 2] = std::env::args()
            .collect::<Vec<_>>().try_into()
            .unwrap_or_else(|_| panic!("expected 1 argument!"));

        let string = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("could not read file {path}: {e}"));

        let debug_log = debug::open_file().expect("could not open log file");
        debugln!(debug_log, "{path}");

        let mut block = parse(string)?;
        debugln!(debug_log, "{block:#?}");

        let builtins = stdlib::get_builtins();
        let mut analysis = Analysis::default();
        let mut functions = HashMap::new();
        let main_function = analysis.next_id();
        assert_eq!(main_function, 0);
        functions.insert(main_function, Vec::new());
        let mut initial_scope = HashMap::new();

        for (ident, (func, sig)) in builtins {
            let id = analysis.next_id();
            debugln!(debug_log, "assign id: builtin {ident} -> {id}");
            initial_scope.insert(ident, id);
            functions.insert(id, func);
            analysis.types.insert(id, Type::Function(sig));
        }

        let analysis = analysis::execute(
            debug_log.clone(),
            &mut block,
            initial_scope,
            analysis
        )?;

        debugln!(debug_log, "{analysis:#?}");

        let mut generator = CodeGenerator::new(debug_log.clone(), functions, analysis);
        let Node::Block(block) = block else { panic!() };
        block.generate(&mut generator)?;
        generator.do_final()?;

        #[cfg(feature = "debug-logging")]
        {
            for (i, func) in &generator.functions {
                debugln!(debug_log, "function {i}:");
                for i in 0..func.len() {
                    debugln!(debug_log, "    {i}: {:?}", func[i]);
                }
            }
        }

        let mut runtime = Runtime::new(debug_log.clone(), generator.functions);
        runtime.call(main_function);

        Ok(())
    }
}
