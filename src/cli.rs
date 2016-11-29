use bench;
use docopt::Docopt;
use env_logger;
use std::env;
use std::io;
use std::io::prelude::*;
use std::process;

const USAGE: &'static str = "
Execute `cargo bench`, recording the results for later analysis.

Usage:
    cargo-metro bench [<bench-option>...]

Running benchmarks:
";

// dead code allowed for now
#[allow(dead_code)]
#[derive(RustcDecodable)]
pub struct Args {
    cmd_bench: bool,
    arg_bench_option: Vec<String>,
}

pub fn main() {
    env_logger::init().unwrap();
    debug!("env_logger initialized");

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(env::args().into_iter()).decode())
        .unwrap_or_else(|e| e.exit());

    let mut stderr = io::stderr();
    if args.cmd_bench {
        match bench::bench(&args.arg_bench_option) {
            Ok(()) => { }
            Err(err) => {
                writeln!(&mut stderr, "error: {}", err).unwrap();
                process::exit(1);
            }
        }
    }
}

