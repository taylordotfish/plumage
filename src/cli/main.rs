/*
 * Copyright (C) 2023, 2025 taylor.fish <contact@taylor.fish>
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
use std::io::{BufReader, BufWriter, Read, Write};

const USAGE: &str = "\
Usage: plumage <name>

Creates `<name>.bmp` and `<name>.params`.
Optionally reads params from `./params`.
";

#[macro_use]
mod error;

fn deserialize_params<R: Read>(stream: R) -> Params {
    ron::de::from_reader(stream).unwrap_or_else(|e| {
        error_exit!("could not read ./params: {e}");
    })
}

fn params_write_failed<T>(e: impl Display) -> T {
    error_exit!("could not write to output params file: {e}");
}

fn main() {
    let mut name = None;
    let mut options_done = false;
    for arg in env::args().skip(1) {
        if options_done {
        } else if arg == "--help" || arg == "-h" {
            print!("{USAGE}");
            return;
        } else if arg == "--version" {
            println!("{}", env!("CARGO_PKG_VERSION"));
            return;
        } else if arg == "--" {
            options_done = true;
            continue;
        } else if arg.starts_with('-') {
            args_error!("unrecognized option: {arg}");
        }
        if name.is_some() {
            args_error!("unexpected argument: {arg}");
        }
        name = Some(arg);
    }
    let Some(mut name) = name else {
        args_error!("missing <name>");
    };

    // Read input params.
    let params = if let Ok(f) = File::open("params") {
        deserialize_params(BufReader::new(f))
    } else {
        deserialize_params("()".as_bytes())
    };

    // Create output params file.
    let name_len = name.len();
    name.push_str(".params");
    let file = File::create(&name).unwrap_or_else(|e| {
        error_exit!("could not create output params file: {e}");
    });
    let mut writer = BufWriter::new(file);
    let pretty = PrettyConfig::new().depth_limit(1);
    ron::ser::to_writer_pretty(&mut writer, &params, pretty)
        .unwrap_or_else(params_write_failed);
    writeln!(writer)
        .and_then(|_| writer.flush())
        .unwrap_or_else(params_write_failed);
    drop(writer);

    // Create image.
    name.replace_range(name_len.., ".bmp");
    let generator = Generator::new(params);
    let file = File::create(name).unwrap_or_else(|e| {
        error_exit!("could not create output file: {e}");
    });
    let mut writer = BufWriter::new(file);
    generator
        .generate(&mut writer)
        .and_then(|_| writer.flush())
        .unwrap_or_else(|e| {
            error_exit!("error generating image: {e}");
        });
}
