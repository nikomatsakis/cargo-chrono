extern crate chrono;
extern crate csv;
extern crate docopt;
#[macro_use]
extern crate error_chain;
extern crate env_logger;
extern crate glob;
extern crate git2;
extern crate gnuplot;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate rustc_serialize;
extern crate pbr;

#[macro_use]
mod macros;

mod bench;
mod cli;
mod data;
mod errors;
mod git;
mod plot;

pub use cli::main;
