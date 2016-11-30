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
mod macros;

mod bench;
mod cli;
mod errors;
mod git;

pub use cli::main;
