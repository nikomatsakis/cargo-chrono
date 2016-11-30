error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Git2(::git2::Git2Error);
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
        return Err($expr.into());
    }
}
