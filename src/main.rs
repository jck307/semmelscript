use semmel::*;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
    }
}
