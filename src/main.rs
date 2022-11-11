// use crate::prelude::*;

// mod error;
// mod prelude;


use clap::Parser;
use time_for::{clapper};

fn main() -> anyhow::Result<()> {
    // errors()?;
    let res = time_for::run(clapper::Inputs::parse());
    if let Err(e) = res {
        let e: anyhow::Error = e.into();
        eprintln!("\nError: {:?}", e);
        std::process::exit(1);
    }
    Ok(())
}
