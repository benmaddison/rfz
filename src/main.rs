extern crate clap;
extern crate directories;
extern crate kuchiki;
extern crate lazycell;
extern crate num_cpus;
extern crate pipeliner;

mod cli;
mod cmd;
mod document;
mod document_set;
mod errors;

#[cfg(test)]
mod test;

use cli::{Cli, Defaults};
use errors::DocumentError;

fn main() -> Result<(), DocumentError> {
    let defaults = Defaults::get()?;
    let cli = Cli::init(&defaults);
    cli.run()
}
