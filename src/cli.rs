use errors::*;
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
    cargo-chrono [options] bench [--label <label>] [<bench-option>...]

Running benchmarks:

Options:
    -f, --file <file>            Data file to write to [default: chrono.csv].
";

// dead code allowed for now
#[allow(dead_code)]
#[derive(RustcDecodable)]
pub struct Args {
    cmd_bench: bool,
    arg_bench_option: Vec<String>,
    flag_file: String,
    flag_label: Option<String>,
}

pub fn main() {
    if let Err(ref e) = run() {
        println_err!("error: {}", e);

        for e in e.iter().skip(1) {
            println_err!("caused by: {}", e);
        }

        process::exit(1);
    }
}

fn run() -> Result<()> {
    env_logger::init().unwrap();
    debug!("env_logger initialized");

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(env::args().into_iter()).decode())
        .unwrap_or_else(|e| e.exit());

    if args.cmd_bench {
        bench::bench(&args.flag_file,
                     &args.flag_label,
                     &args.arg_bench_option)?;
    }

    Ok(())
}
