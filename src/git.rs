use std::io::{self, Write};
use std::path::Path;
use git2::{Commit, Error as Git2Error, ErrorCode, Object, Repository, Status,
           STATUS_IGNORED};
use errors::*;

/// Search upwards from `start_path` to find a valid git repo.
pub fn open_repo(cargo_path: &Path) -> Result<Repository> {
    let mut git_path = cargo_path;

    loop {
        if git_path.is_dir() {
            match Repository::open(git_path) {
                Ok(r) => {
                    return Ok(r);
                }
                Err(err) => {
                    match err.code() {
                        ErrorCode::NotFound => {}
                        _ => {
                            throw!(err);
                        }
                    }
                }
            }
        }

        git_path = match git_path.parent() {
            Some(p) => p,
            None => return Ok(Repository::open(cargo_path)?),
        }
    }
}

pub fn check_clean(repo: &Repository) -> Result<()> {
    let statuses = repo.statuses(None)
        .chain_err(|| "could not load git repository status")?;

    let mut errors = 0;
    let dirty_status = Status::all() - STATUS_IGNORED;
    for status in statuses.iter() {
        if status.status().intersects(dirty_status) {
            let stderr = io::stderr();
            let mut stderr = stderr.lock();
            if let Some(p) = status.path() {
                writeln!(stderr, "file `{}` is dirty", p).unwrap();
            }
            errors += 1;
        }
    }
    if errors > 0 {
        throw!(ErrorKind::DirtyRepo(errors))
    }

    Ok(())
}

