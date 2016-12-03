use errors::*;
use bench;
use docopt::Docopt;
use env_logger;
use plot;
use std::env;
use std::process;

const USAGE: &'static str = "
Execute `cargo bench`, recording the results for later analysis.

Usage:
    cargo-chrono bench [options] [--] [<bench-option>...]
    cargo-chrono plot [options] [<plot-filter>...]
    cargo-chrono --help

How to use it.

Options:
    -f, --file <file>            Data file to write to [default: chrono.csv].
    --commits <commit-list>      (bench:) check out each commit in the (space-separated) list
                                 in turn and run the benchmark, accumulating results
    --ignore-dirty <glob>        (bench:) Ignore dirty files that match the given glob pattern.
    --repeat <N>                 (bench:) Take N measurements when benchmarking [default: 1].
    --include-variance           (plot:) Include variance as errors bars.
    --medians                    (plot:) Plot medians of all samples (with error bars).
    --output-file <file>         (plot:) Where to write the output [default: chrono.svg].
";

// dead code allowed for now
#[allow(dead_code)]
#[derive(RustcDecodable)]
pub struct Args {
    cmd_bench: bool,
    cmd_plot: bool,
    arg_bench_option: Vec<String>,
    arg_plot_filter: Vec<String>,
    flag_file: String,
    flag_repeat: usize,
    flag_ignore_dirty: Vec<String>,
    flag_include_variance: bool,
    flag_medians: bool,
    flag_output_file: String,
    flag_commits: Option<String>,
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
                     &args.flag_ignore_dirty,
                     args.flag_repeat,
                     &args.flag_commits,
                     &args.arg_bench_option)?;
    } else if args.cmd_plot {
        plot::plot(&args.flag_file,
                   plot::Config {
                       include_variance: args.flag_include_variance,
                       compute_medians: args.flag_medians,
                       output_file: &args.flag_output_file,
                       filters: &args.arg_plot_filter,
                   })?;
    } else {
        throw!("bug: unknown command")
    }

    Ok(())
}
