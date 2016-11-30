extern crate docopt;
#[macro_use]
extern crate error_chain;
extern crate env_logger;
extern crate git2;
#[macro_use]
extern crate log;
extern crate regex;
extern crate rustc_serialize;

#[macro_use]
mod errors;

mod bench;
mod cli;
mod git;

pub use cli::main;
