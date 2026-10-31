# Third-party notices

Mouse Tester is distributed under the MIT license (see `LICENSE`).

This file lists the Rust crates included in the Linux x86_64 build. Where a
crate offers a choice (for example, MIT OR Apache-2.0), this distribution relies
on the MIT option unless noted otherwise. The corresponding license texts are
in `licenses/`.

| Crate | Version | License |
| --- | --- | --- |
| `allocator-api2` | 0.2.21 | MIT OR Apache-2.0 |
| `anstream` | 1.0.0 | MIT OR Apache-2.0 |
| `anstyle` | 1.0.14 | MIT OR Apache-2.0 |
| `anstyle-parse` | 1.0.0 | MIT OR Apache-2.0 |
| `anstyle-query` | 1.1.5 | MIT OR Apache-2.0 |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 |
| `cassowary` | 0.3.0 | MIT / Apache-2.0 |
| `castaway` | 0.2.4 | MIT |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 |
| `clap` | 4.6.7 | MIT OR Apache-2.0 |
| `clap_builder` | 4.6.7 | MIT OR Apache-2.0 |
| `clap_derive` | 4.6.7 | MIT OR Apache-2.0 |
| `clap_lex` | 1.1.1 | MIT OR Apache-2.0 |
| `colorchoice` | 1.0.5 | MIT OR Apache-2.0 |
| `compact_str` | 0.8.2 | MIT |
| `crossterm` | 0.28.1 | MIT |
| `darling` | 0.24.1 | MIT |
| `darling_core` | 0.24.1 | MIT |
| `darling_macro` | 0.24.1 | MIT |
| `either` | 1.19.0 | MIT OR Apache-2.0 |
| `equivalent` | 1.0.3 | Apache-2.0 OR MIT |
| `errno` | 0.3.14 | MIT OR Apache-2.0 |
| `foldhash` | 0.1.5 | Zlib |
| `hashbrown` | 0.15.5 | MIT OR Apache-2.0 |
| `heck` | 0.5.0 | MIT OR Apache-2.0 |
| `ident_case` | 1.0.1 | MIT/Apache-2.0 |
| `indoc` | 2.0.8 | MIT OR Apache-2.0 |
| `instability` | 0.3.14 | MIT |
| `is_terminal_polyfill` | 1.70.2 | MIT OR Apache-2.0 |
| `itertools` | 0.13.0 | MIT OR Apache-2.0 |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 |
| `libc` | 0.2.191 | MIT OR Apache-2.0 |
| `linux-raw-sys` | 0.4.15 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 |
| `log` | 0.4.35 | MIT OR Apache-2.0 |
| `lru` | 0.12.5 | MIT |
| `mio` | 1.2.4 | MIT |
| `mouse_tester` | 1.0.0 (/home/daniel/myDev/my_own_old_games/mouse) | MIT |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 |
| `paste` | 1.0.15 | MIT OR Apache-2.0 |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 |
| `quote` | 1.0.47 | MIT OR Apache-2.0 |
| `ratatui` | 0.29.0 | MIT |
| `rustix` | 0.38.44 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 |
| `ryu` | 1.0.23 | Apache-2.0 OR BSL-1.0 |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 |
| `signal-hook` | 0.3.18 | Apache-2.0/MIT |
| `signal-hook-mio` | 0.2.5 | MIT OR Apache-2.0 |
| `signal-hook-registry` | 1.4.8 | MIT OR Apache-2.0 |
| `smallvec` | 1.16.4 | MIT OR Apache-2.0 |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 |
| `strsim` | 0.11.1 | MIT |
| `strum` | 0.26.3 | MIT |
| `strum_macros` | 0.26.4 | MIT |
| `syn` | 2.0.119 | MIT OR Apache-2.0 |
| `syn` | 3.0.9 | MIT OR Apache-2.0 |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 |
| `unicode-truncate` | 1.1.0 | MIT OR Apache-2.0 |
| `unicode-width` | 0.1.14 | MIT OR Apache-2.0 |
| `unicode-width` | 0.2.0 | MIT OR Apache-2.0 |
| `utf8parse` | 0.2.2 | Apache-2.0 OR MIT |

## License texts

- `licenses/MIT.txt` — MIT
- `licenses/Apache-2.0.txt` — Apache License 2.0
- `licenses/Apache-2.0-LLVM-exception.txt` — Apache 2.0 with LLVM exception
- `licenses/BSL-1.0.txt` — Boost Software License 1.0
- `licenses/Unicode-3.0.txt` — Unicode License v3
- `licenses/Zlib.txt` — zlib License

The Windows-only crates present in `Cargo.lock` are not part of the Linux binary.
