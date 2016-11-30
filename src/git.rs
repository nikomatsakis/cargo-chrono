use git2::{Commit, Error as Git2Error, ErrorCode, Object, Repository, Status,
           STATUS_IGNORED};
use errors::{ErrorKind, Result};

/// Search upwards from `start_path` to find a valid git repo.
pub fn open_repo(start_path: &Path) -> Result<Repository> {
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
                            return Err(err);
                        }
                    }
                }
            }
        }

        git_path = match git_path.parent() {
            Some(p) => p,
            None => return Repository::open(cargo_path),
        }
    }
}

pub fn check_clean(repo: &Repository) -> Result<()> {
    let statuses = match repo.statuses(None) {
        Ok(s) => s,
        Err(err) => error!("could not load git repository status: {}", err),
    };

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
}

