extern crate docopt;
extern crate git2;
extern crate regex;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate env_logger;

mod bench;
mod cli;

pub use cli::main;
