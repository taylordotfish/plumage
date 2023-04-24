/*
 * Copyright (C) 2023 taylor.fish <contact@taylor.fish>
 *
 * This file is part of Plumage.
 *
 * Plumage is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Plumage is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with Plumage. If not, see <https://www.gnu.org/licenses/>.
 */

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
