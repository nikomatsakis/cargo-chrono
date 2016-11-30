macro_rules! throw {
    ($e:expr) => {
        return Err($e.into());
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(format!($fmt, $($arg)+).into());
    };
}

macro_rules! println_err {
    ($fmt:expr, $($arg:tt)+) => {
        writeln!(&mut io::stderr(), $fmt, $($arg)+).unwrap()
    };
}
