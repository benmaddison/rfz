extern crate clap;
extern crate directories;
extern crate kuchiki;
extern crate lazycell;
extern crate num_cpus;
extern crate pipeliner;

mod cli;
mod cmd;
mod collection;
mod document;
mod errors;

#[cfg(test)]
mod test;

pub use cli::{Cli, Defaults};
pub use errors::Result;
