use errors::*;
use csv;

#[derive(Clone, Debug, RustcDecodable)]
pub struct Measurement {
    pub commit: String, // a sha1 hash
    pub test: String, // name of test that was run
    pub time: u64,
    pub variance: u64,
}

pub fn load_measurements(path: &str) -> Result<Vec<Measurement>> {
    let mut reader = csv::Reader::from_file(&path)
        .chain_err(|| format!("cannot read `{}`", path))?;

    reader.decode()
          .map(|r| r.chain_err(|| format!("cannot decode CSV data")))
          .collect()
}
