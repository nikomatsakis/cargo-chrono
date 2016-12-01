error_chain! {
    errors {
        DirtyRepo(errors: usize) {
            description("repository contains dirty files")
            display("repository contains {} dirty files", errors)
        }
    }
}
