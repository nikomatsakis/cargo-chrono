use git;
use regex;
use std::env;
use std::error::Error;
use std::process::Command;

pub fn bench(options: &[String]) -> Result<(), Box<Error>> {
    let current_dir = env::current_dir()?;
    let repo = git::open_repo(&current_dir)?;
    
    let mut output = Command::new("cargo");
    output.arg("bench");
    for option in options {
        output.arg(option);
    }
    let output = output.output()?;
    println!("{:?}", output);
    Ok(())
}
