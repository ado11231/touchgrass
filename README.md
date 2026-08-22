# touchgrass

Track active time in your dev tools and view it as a terminal heatmap.
Runs on macOS and Linux. Time stays on your computer.

Requires Rust 1.95 or newer.

```sh
cargo install --path . --locked
touchgrass install
touchgrass
```

```text
touchgrass          Show recent time
touchgrass 2026     Show one calendar year
touchgrass track    Track in the current terminal
touchgrass install  Start now and at login
touchgrass uninstall
                    Stop the login service and keep saved time
```

While you are active, the tracker credits the focused app (kitty, VS Code)
and the best tool running inside it (Claude Code in a kitty tab, Neovim).
Untracked apps, like a browser, count nothing.
Edit `config.toml` to change the tool list or idle limit.

Linux autostart needs a systemd user session. Idle checks use `xprintidle`
on X11 or `gdbus` on GNOME Wayland. If idle checks are unavailable, the
tracker warns once and counts as active.

- [Commands and output](docs/COMMANDS.md)
- [Design, settings, and saved files](docs/ARCHITECTURE.md)
- [Contributing](docs/CONTRIBUTING.md)

[MIT license](LICENSE).
