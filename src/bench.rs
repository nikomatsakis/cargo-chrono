use errors::*;
use git;
use std::env;
use std::process::Command;

pub fn bench(data_file: &String,
             opt_run_label: &Option<String>,
             bench_options: &[String]) -> Result<()> {
    let run_label = if let Some(ref l) = *opt_run_label {
        l.clone()
    } else {
        // repository must be clean
        let current_dir = env::current_dir()?;
        let repo = git::open_repo(&current_dir)?;
        git::check_clean(&repo)?;

        // find the current commit sha1 hash
        let s = repo.head()?.target().unwrap().to_string();
        s
    };

    let mut output = Command::new("cargo");
    output.arg("bench");
    for bench_option in bench_options {
        output.arg(bench_option);
    }
    let output = output.output()?;
    println!("{:?}", output);
    Ok(())
}
