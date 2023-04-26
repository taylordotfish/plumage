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

#![deny(unsafe_op_in_unsafe_fn)]

use plumage::{Generator, Params};
use ron::ser::PrettyConfig;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;

const USAGE: &str = "\
Usage: plumage <name>

Creates `<name>.bmp` and `<name>.params`.
Optionally reads params from `./params`.
";

#[macro_use]
mod error;

fn deserialize_params<R: Read>(stream: R) -> Params {
    ron::de::from_reader(stream).unwrap_or_else(|e| {
        error_exit!("error reading params: {e}");
    })
}

fn usage() {
    print!("{USAGE}");
    exit(0);
}

fn params_write_failed<T>(e: impl Display) -> T {
    error_exit!("could not write to output params file: {e}");
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(mut name) = args.next() else {
        args_error!("missing <name>");
    };
    if let Some(arg) = args.next() {
        args_error!("unexpected argument: {arg}");
    }
    let name_len = name.len();
    if name == "-h" || name == "--help" {
        usage();
    }

    // Read input params.
    let params = if let Ok(f) = File::open("params") {
        deserialize_params(f)
    } else {
        deserialize_params("()".as_bytes())
    };

    // Create output params file.
    name.replace_range(name_len.., ".params");
    let mut f = File::create(&name).unwrap_or_else(|e| {
        error_exit!("could not create output params file: {e}");
    });
    let pretty = PrettyConfig::new().depth_limit(1);
    ron::ser::to_writer_pretty(&mut f, &params, pretty)
        .unwrap_or_else(params_write_failed);
    writeln!(f).unwrap_or_else(params_write_failed);
    drop(f);

    // Create image.
    name.replace_range(name_len.., ".bmp");
    let generator = Generator::new(params);
    let f = File::create(name).unwrap_or_else(|e| {
        error_exit!("could not create output file: {e}");
    });
    generator.generate(f).unwrap_or_else(|e| {
        error_exit!("error generating image: {e}");
    });
}
