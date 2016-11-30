error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Git2(::git2::Error);
    }

    errors {
        DirtyRepo(errors: usize) {
            description("repository contains dirty files")
            display("repository contains {} dirty files", errors)
        }
    }
}

macro_rules! throw {
    ($e:expr) => {
        return Err($e.into());
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(format!($fmt, $($arg)+).into());
    };
}
