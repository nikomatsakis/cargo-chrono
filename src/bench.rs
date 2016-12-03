use csv;
use errors::*;
use git;
use git2::{ObjectType, Repository};
use git2::build::CheckoutBuilder;
use glob;
use pbr::ProgressBar;
use regex::Regex;
use std::env;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::Command;
use std::io::prelude::*;
use std::str;

lazy_static! {
    // Example:
    // test nbody::bench::nbody_par              ... bench:  12,459,703 ns/iter (+/- 75,027)
    pub static ref BENCH_RE: Regex = Regex::new(
        r"\s*test\s+([^ ]+)\s*...\s*bench:\s*([0-9,]+) ns/iter \(\+/- ([0-9,]+)\)\s*").unwrap();
}

pub fn bench(data_file: &str,
             ignore_dirty: &[String],
             flag_repeat: usize,
             commits: &Option<String>,
             bench_options: &[String])
             -> Result<()> {
    let data_path: &Path = Path::new(data_file);

    // Find the files that match the ignore patterns.
    let mut ignored_paths = vec![data_path.to_owned()];
    for pattern in ignore_dirty {
        let paths = glob::glob(pattern).chain_err(|| format!("invalid glob pattern: `{}`", pattern))?;
        for path in paths {
            let path = path.chain_err(|| format!("error accessing path for pattern `{}`", pattern))?;
            ignored_paths.push(path);
        }
    }

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
    git::check_clean(&repo, &ignored_paths)?;
    let head = repo.head().chain_err(|| "failed to fetch HEAD from repo")?;

    // Parse the `bench_options` and separate them into benchmark names (no leading `-`)
    // and flags.
    let (bench_flags, mut bench_names): (Vec<_>, Vec<_>) = bench_options.iter()
        .cloned()
        .partition(|s| s.starts_with("-"));

    if bench_names.is_empty() {
        bench_names.push(String::new());
    }

    let runs_per_commit = flag_repeat * bench_names.len() + 1;

    let mut bar = ProgressBar::new(runs_per_commit as u64);
    bar.show_speed = false;
    bar.show_counter = false;
    bar.show_time_left = false;
    bar.show_tick = false;
    bar.show_message = true;

    // if the user gave us a list of commits, check out each one in turn
    let mut writer = csv::Writer::from_writer(data_file);
    if let Some(ref commits_str) = *commits {
        // let users write "a,b" or "a b"
        let head_commit = head.peel(ObjectType::Commit)
            .chain_err(|| "HEAD not a commit")?;
        let revisions: Vec<_> = try!(commits_str.split(",")
            .flat_map(|s| s.split_whitespace())
            .map(|c| repo.revparse_single(c).chain_err(|| format!("invalid revision '{}'", c)))
            .collect());
        if let Some(r) = revisions.iter().find(|r| r.as_commit().is_none()) {
            bail!("revision `{}` is not a commit", git::short_id(r));
        }
        let total_commits = revisions.len();
        bar.total *= total_commits as u64;
        for commit in revisions.iter().filter_map(|r| r.as_commit()) {
            bar.message(&format!("checking out `{}`", git::short_id(commit)));
            git::checkout_commit(&repo, commit)
                .chain_err(|| format!("failed to checkout commit `{}`", git::short_id(commit)))?;
            run_bench(&mut bar,
                      &repo,
                      &mut writer,
                      &bench_flags,
                      &bench_names,
                      flag_repeat)
                ?;
        }
        bar.message("restoring HEAD");
        repo.checkout_tree(&head_commit, Some(&mut CheckoutBuilder::new()))
            .chain_err(|| {
                format!("failed to checkout original HEAD `{}`",
                        git::short_id(&head_commit))
            })?;
        let name = head.name().ok_or("HEAD not utf-8")?;
        repo.set_head(name)
            .chain_err(|| format!("failed to restore original HEAD `{}`", name))?;
    } else {
        run_bench(&mut bar,
                  &repo,
                  &mut writer,
                  &bench_flags,
                  &bench_names,
                  flag_repeat)
            ?;
    }

    Ok(())
}

fn run_bench<F, WB>(bar: &mut ProgressBar<WB>,
                    repo: &Repository,
                    writer: &mut csv::Writer<F>,
                    bench_flags: &[String],
                    bench_names: &[String],
                    flag_repeat: usize)
                    -> Result<()>
    where F: Write,
          WB: Write
{
    // how many total times will we run cargo
    let mut tick = |title: &str| {
        bar.message(title);
        bar.inc();
    };

    // find the current commit sha1 hash
    let commit = git::short_id(&repo.head()
        .chain_err(|| "failed to fetch HEAD from repo")?
        .peel(ObjectType::Commit)
        .chain_err(|| "HEAD not a commit")?);

    {
        tick(&format!("building `{}`", commit));
        let mut cargo = Command::new("cargo");
        cargo.arg("bench");
        for bench_flag in bench_flags {
            cargo.arg(bench_flag);
        }
        cargo.arg("--no-run");
        let output = cargo.output().chain_err(|| "error executing `cargo bench`")?;
        if !output.status.success() {
            bail!("`{:?}` exited with error-code `{}`", cargo, output.status);
        }
    }

    // for each benchmark name they gave us...
    for bench_name in bench_names {
        // repeat N times...
        for i in 0..flag_repeat {
            // ...run cargo and save the output.
            if !bench_name.is_empty() {
                tick(&format!("testing `{}` from `{}` (run {}/{})",
                              bench_name,
                              commit,
                              i + 1,
                              flag_repeat));
            } else {
                tick(&format!("testing `{}` (run {}/{})", commit, i + 1, flag_repeat));
            }
            let mut cargo = Command::new("cargo");
            cargo.arg("bench");
            for bench_flag in bench_flags {
                cargo.arg(bench_flag);
            }
            if !bench_name.is_empty() {
                cargo.arg(bench_name);
            }
            let output = cargo.output()
                .chain_err(|| "error executing `cargo bench`")?;
            if !output.status.success() {
                bail!("`{:?}` exited with error-code `{}`", cargo, output.status);
            }
            let output_str = match str::from_utf8(&output.stdout) {
                Ok(s) => s,
                Err(_) => throw!("`cargo bench` did not output utf-8"),
            };

            // Grep through the output and collect new data, appending it to
            // the data file as we go. The data has this format:
            //
            // (label, test_name, time, variance)
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
    }

    Ok(())
}
