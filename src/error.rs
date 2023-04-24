use std::fmt::Display;
use std::process::exit;

/// Displays an error message and exits.
macro_rules! error_exit {
    ($($args:tt)*) => {
        crate::error::__exit(format_args!($($args)*))
    };
}

macro_rules! args_error {
    ($($args:tt)*) => {
        error_exit!(
            "{}\n{}",
            format_args!($($args)*),
            "See `plumage --help` for usage information.",
        );
    };
}

#[doc(hidden)]
pub fn __exit(args: impl Display) -> ! {
    eprintln!("error: {args}");
    if cfg!(feature = "panic") {
        panic!("error: {args}");
    } else {
        exit(1);
    }
}
