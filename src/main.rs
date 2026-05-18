use semmel::{*, runtime::Evaluate};

fn run() -> Result<()> {
    let [_, path]: [String; 2] = std::env::args()
        .collect::<Vec<_>>().try_into()
        .unwrap_or_else(|_| panic!("Expected 1 argument!"));

    let string = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Could not read file {path}: {e}"));

    let block = parse(string)?;
    let (mut runtime, mut scope) = setup();
    set_runtime_pointer(&mut runtime, &mut scope);
    block.eval(&mut runtime, &mut scope)?;

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
    }
}
