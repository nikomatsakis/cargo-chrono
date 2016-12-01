use csv;
use errors::*;
use git;
use git2::ObjectType;
use regex::Regex;
use std::env;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::Command;
use std::str;

lazy_static! {
    // Example:
    // test nbody::bench::nbody_par              ... bench:  12,459,703 ns/iter (+/- 75,027)
    pub static ref BENCH_RE: Regex = Regex::new(
        r"\s*test\s+([^ ]+)\s*...\s*bench:\s*([0-9,]+) ns/iter \(\+/- ([0-9,]+)\)\s*").unwrap();
}

pub fn bench(data_file: &str,
             ignore_dirty: bool,
             flag_repeat: usize,
             bench_options: &[String])
             -> Result<()> {
    let data_path: &Path = Path::new(data_file);

    // Open the data file for append early, so that we detect errors
    // *before* we run cargo bench.
    let data_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&data_path)
        .chain_err(|| format!("failed to open data file `{}`", data_path.display()))?;

    // Find the current commit. Check that repository is clean.
    let current_dir = env::current_dir().chain_err(|| "failed to find current dir")?;
    let repo = git::open_repo(&current_dir).chain_err(|| "failed to open git repo")?;
    match git::check_clean(&repo, &[data_path]) {
        Ok(()) => {}
        Err(_) if ignore_dirty => {}
        Err(err) => throw!(err),
    }

    // find the current commit sha1 hash
    let commit = git::short_id(&repo.head()
        .chain_err(|| "failed to fetch HEAD from repo")?
        .peel(ObjectType::Commit)
        .chain_err(|| "HEAD not a commit")?);

    for _ in 0..flag_repeat {
        // Execute cargo and save the output.
        let mut cargo = Command::new("cargo");
        cargo.arg("bench");
        for bench_option in bench_options {
            cargo.arg(bench_option);
        }
        let output = cargo.output()
            .chain_err(|| "error executing `cargo bench`")?;
        if !output.status.success() {
            throw!("`cargo bench` exited with error-code `{}`", output.status);
        }
        let output_str = match str::from_utf8(&output.stdout) {
            Ok(s) => s,
            Err(_) => throw!("`cargo bench` did not output utf-8"),
        };

        // Grep through the output and collect new data, appending it to
        // the data file as we go. The data has this format:
        //
        // (label, test_name, time, variance)
        let mut writer = csv::Writer::from_writer(&data_file);
        for line in output_str.lines() {
            if let Some(captures) = BENCH_RE.captures(line) {
                let (name, time_str, variance_str) = (&captures[1], &captures[2], &captures[3]);
                let time_str: String = time_str.chars().filter(|&c| c != ',').collect();
                let variance_str: String = variance_str.chars().filter(|&c| c != ',').collect();
                let data = (&commit, name, time_str, variance_str);
                writer.encode(data)
                    .chain_err(|| format!("failed to write data for test `{}`", name))?;
            }
        }
    }

    Ok(())
}
