use std::io::{self, Write};
use std::path::Path;
use git2::{Commit, ErrorCode, Object, Repository, Status, STATUS_IGNORED};
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
                            Err(err).chain_err(|| "failed to open git repository")?;
                        }
                    }
                }
            }
        }

        git_path = match git_path.parent() {
            Some(p) => p,
            None => {
                return Ok(Repository::open(cargo_path)
                    .chain_err(|| "failed to open git repository")?)
            }
        }
    }
}

pub fn check_clean(repo: &Repository, exceptions: &[&Path]) -> Result<()> {
    let statuses = repo.statuses(None)
        .chain_err(|| "could not load git repository status")?;

    let exceptions: Vec<_> = try!(exceptions.iter()
        .map(|e| {
            e.canonicalize().chain_err(|| format!("failed to canonicalize `{}`", e.display()))
        })
        .collect());

    let workdir = match repo.workdir() {
        Some(w) => w,
        None => throw!("bare repositories are not supported"),
    };
    let mut errors = 0;
    let dirty_status = Status::all() - STATUS_IGNORED;
    for status in statuses.iter() {
        if status.status().intersects(dirty_status) {
            let stderr = io::stderr();
            let mut stderr = stderr.lock();
            if let Some(path_str) = status.path() {
                let path = workdir.join(path_str)
                    .canonicalize()
                    .chain_err(|| format!("failed to canonicalize `{}`", path_str))?;
                if !exceptions.iter().any(|e| *e == path) {
                    writeln!(stderr, "file `{}` is dirty", path_str).unwrap();
                }
            }
            errors += 1;
        }
    }
    if errors > 0 {
        throw!(ErrorKind::DirtyRepo(errors))
    }

    Ok(())
}

pub trait AsObject<'repo> {
    fn as_object(&self) -> &Object<'repo>;
}

impl<'repo> AsObject<'repo> for Object<'repo> {
    fn as_object(&self) -> &Object<'repo> {
        self
    }
}

impl<'repo> AsObject<'repo> for Commit<'repo> {
    fn as_object(&self) -> &Object<'repo> {
        self.as_object()
    }
}

// pub fn short_id<'repo, T>(obj: &T) -> String
//    where T: AsObject<'repo>
// {
//    let obj = obj.as_object();
//    match obj.short_id() {
//        Ok(buf) => buf.as_str().unwrap().to_string(), // should really be utf-8
//        Err(_) => obj.id().to_string(), // oh screw it use the full id
//    }
// }
//
// pub fn commit_or_error<'obj, 'repo>(obj: Object<'repo>) -> Result<Commit<'repo>> {
//    match obj.into_commit() {
//        Ok(commit) => Ok(commit),
//        Err(obj) => throw!("object `{}` is not a commit", short_id(&obj)),
//    }
// }
