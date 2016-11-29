use regex;
use std::error::Error;
use std::process::Command;

pub fn bench(options: &[String]) -> Result<(), Box<Error>> {
    let mut output = Command::new("cargo");
    output.arg("bench");
    for option in options {
        output.arg(option);
    }
    let output = output.output()?;
    println!("{:?}", output);
    Ok(())
}
