# chocodye

A Rust library for changing the color of the chocobos' plumage in *Final Fantasy XIV*.

[![Current version](https://img.shields.io/crates/v/chocodye)](https://crates.io/crates/chocodye/)
[![License](https://img.shields.io/crates/l/chocodye)](#license)
[![Crate size](https://img.shields.io/badge/crate%20size-28.9%20KiB-blue)]()
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.66-blue)](https://blog.rust-lang.org/2022/12/15/Rust-1.66.0.html)
[![Documentation](https://img.shields.io/docsrs/chocodye)](https://docs.rs/chocodye/)
[![Maintained](https://img.shields.io/maintenance/yes/2023)]()

This repository also includes language-agnostic files such as [`dyes.xml`](src/xml/dyes.xml)
and [Fluent translation lists](src/ftl/).

## Documentation

Latest version:
https://docs.rs/chocodye/

## Cargo Features

- `fluent`: enables localization through [Fluent](https://projectfluent.org/).
- `truecolor`: enables text to be colored in the terminal.

## Examples

List of colors:
```bash
cargo run --example truecolor -- en
```

![Result of running the truecolor example](https://github.com/ShE3py/chocodye/blob/4898eb80cf600dc3e179a4758ba90e3a060bafdd/examples/truecolor.png?raw=true)

Sample menu:

```bash
cargo run --example menu -- en
```

![Result of running the menu example](https://github.com/ShE3py/chocodye/blob/4898eb80cf600dc3e179a4758ba90e3a060bafdd/examples/menu.png?raw=true)

The displayed language can be changed by replacing `en` with `fr`, `de` or `jp`.
Please note that for colors to be displayed under Windows, this command must first be run:
```bat
set COLORTERM=truecolor
```
See the documentation for [`chocodye::ansi_text`](https://docs.rs/chocodye/latest/chocodye/fn.ansi_text.html).

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
