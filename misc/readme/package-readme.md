Plumage
=======

Plumage generates colorful pictures.

![example output](https://github.com/taylordotfish/plumage/raw/master/misc/example.png)

Installation
------------

Install with Cargo:

```bash
cargo install plumage
```

Usage
-----

See `plumage --help`. Plumage (optionally) reads parameters from `./params`
(see [params.example]) and generates a bitmap image, saving both the image and
the parameters used to generate it. These parameters can later be used as input
parameters to generate the same image, or they can serve as a starting point
and be tweaked. If any parameters are missing, they will be filled in with
either defaults or random values.

[params.example]: https://github.com/taylordotfish/plumage/blob/master/params.example

For now, Plumage is best suited for technical users who wish to look through
the [source code] or learn by experimentation to discover what the different
parameters do and how the algorithm behaves.

[source code]: https://github.com/taylordotfish/plumage
