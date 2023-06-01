# chocodye

A Rust library for changing the color of the chocobos' plumage in *Final Fantasy XIV*.

[![Current version](https://img.shields.io/crates/v/chocodye)](https://crates.io/crates/chocodye/)
![License](https://img.shields.io/crates/l/chocodye)
![GitHub repo size](https://img.shields.io/github/repo-size/ShE3py/chocodye)
![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.66-blue)
[![Documentation](https://img.shields.io/docsrs/chocodye)](https://docs.rs/chocodye/)
![Maintained](https://img.shields.io/maintenance/yes/2023)

This repository also includes language-agnostic files such as [`dyes.xml`](src/xml/dyes.xml)
and [Fluent translation lists](src/ftl/).

## Documentation

Latest version:
https://docs.rs/chocodye/

## Optional features

- `fluent`: enables localization through [Fluent](https://projectfluent.org/).

## Examples

Sample menu:
```bash
cargo run --example menu --features=fluent -- en
```

List of colors:
```bash
cargo run --example truecolor --features=fluent -- en
```

The displayed language can be changed by replacing `en` with `fr`, `de` or `jp`.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
