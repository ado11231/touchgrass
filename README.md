<div align="center">

### touchgrass

[![check](https://github.com/ado11231/touchgrass/actions/workflows/check.yml/badge.svg)](https://github.com/ado11231/touchgrass/actions/workflows/check.yml)
[![rust](https://img.shields.io/badge/rust-1.95%2B-orange?logo=rust)](https://www.rust-lang.org)
[![platform](https://img.shields.io/badge/platform-macos%20%7C%20linux-lightgrey)](#installation)
[![license](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

track how long you spend in your dev tools

<img src="assets/preview.svg" alt="touchgrass showing a year heatmap and today's time per tool" width="100%">

</div>

### Installation

Requires Rust 1.95 or newer on macOS or Linux.

```sh
git clone https://github.com/ado11231/touchgrass.git
cd touchgrass
cargo install --path . --locked
touchgrass install   # start tracking now and at every login
touchgrass           # show your time
```

On Linux, autostart needs a systemd user session. Idle checks use `xprintidle`
on X11 or `gdbus` on GNOME Wayland.

- [Commands and output](docs/COMMANDS.md)
- [Design, settings, and saved files](docs/ARCHITECTURE.md)
- [Contributing](docs/CONTRIBUTING.md)

[MIT license](LICENSE).
