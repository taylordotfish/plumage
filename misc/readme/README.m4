dnl| Copyright (C) 2024-2025 taylor.fish <contact@taylor.fish>
dnl|
dnl| This file is part of Plumage.
dnl|
dnl| Plumage is free software: you can redistribute it and/or modify
dnl| it under the terms of the GNU Affero General Public License as published
dnl| by the Free Software Foundation, either version 3 of the License, or
dnl| (at your option) any later version.
dnl|
dnl| Plumage is distributed in the hope that it will be useful,
dnl| but WITHOUT ANY WARRANTY; without even the implied warranty of
dnl| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
dnl| GNU Affero General Public License for more details.
dnl|
dnl| You should have received a copy of the GNU Affero General Public License
dnl| along with Plumage. If not, see <https://www.gnu.org/licenses/>.
dnl|
changequote(`{', `}')dnl
define({REPO}, {https://github.com/taylordotfish/plumage})dnl
Plumage
=======

Plumage generates colorful pictures.

![example output](ifdef({RUST}, {REPO/raw/master/})misc/example.png)

Installation
------------

Install with Cargo:

```bash
cargo install plumage
```

ifdef({RUST},, {dnl
Or, clone with Git and compile manually:

```bash
git clone REPO
cd plumage
cargo build --release  # Creates ./target/release/plumage
# Optionally, to install to ~/.cargo/bin:
cargo install --path .
```

})dnl
Usage
-----

See `plumage --help`. Plumage (optionally) reads parameters from `./params`
(see [params.example]) and generates a bitmap image, saving both the image and
the parameters used to generate it. These parameters can later be used as input
parameters to generate the same image, or they can serve as a starting point
and be tweaked. If any parameters are missing, they will be filled in with
either defaults or random values.

[params.example]: ifdef({RUST}, {REPO/blob/master/})params.example

ifdef({RUST},, {dnl
[generate.sh] generates images with Plumage in parallel and converts them to
PNG; see `./generate.sh --help`.

[generate.sh]: generate.sh

})dnl
For now, Plumage is best suited for technical users who wish to look through
the ifdef({RUST},
   {[source code]},
   {source code}) or learn by experimentation to discover what the different
parameters do and how the algorithm behaves.

ifdef({RUST}, {dnl
[source code]: REPO
}, {dnl
License
-------

Plumage is licensed under version 3 of the GNU Affero General Public License,
or (at your option) any later version. See [LICENSE](LICENSE).

Contributing
------------

By contributing to Plumage, you agree that your contribution may be used
according to the terms of Plumage’s license.
})dnl vim-m4: startquote={ endquote=}
