use std::io::{self, Write};
use std::path::{Path, PathBuf};
use git2::{Commit, Object, Repository, Status, STATUS_IGNORED};
use git2::build::CheckoutBuilder;
use errors::*;

/// Search upwards from `start_path` to find a valid git repo.
pub fn open_repo(start_path: &Path) -> Result<Repository> {
    Ok(Repository::discover(start_path)
       .chain_err(|| format!("could not find git repository in `{}`", start_path.display()))?)
}

pub fn check_clean(repo: &Repository, exceptions: &[PathBuf]) -> Result<()> {
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
                if exceptions.iter().any(|e| *e == path) {
                    continue;
                }
                writeln!(stderr, "file `{}` is dirty", path_str).unwrap();
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

pub fn short_id<'repo, T>(obj: &T) -> String
   where T: AsObject<'repo>
{
   let obj = obj.as_object();
   match obj.short_id() {
       Ok(buf) => buf.as_str().unwrap().to_string(), // should really be utf-8
       Err(_) => obj.id().to_string(), // oh screw it use the full id
   }
}

pub fn checkout_commit(repo: &Repository, commit: &Commit)
                       -> Result<()> {
    let mut cb = CheckoutBuilder::new();
    repo.checkout_tree(commit.as_object(), Some(&mut cb))
        .chain_err(|| format!("failed to check out `{}`", short_id(commit)))?;

    repo.set_head_detached(commit.id())
        .chain_err(|| format!("failed to set HEAD to `{}`", short_id(commit)))?;

    Ok(())
}

