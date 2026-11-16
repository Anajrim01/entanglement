use std::error::Error;

#[path = "../build_support.rs"]
mod build_support;
use build_support::GrammarBuild;

fn main() -> Result<(), Box<dyn Error>> {
    let mut build = GrammarBuild::new("tree-sitter-mql5")?;
    build.parser("", |_| Ok(()))?;
    build.finish("entanglement-mql5");
    Ok(())
}
