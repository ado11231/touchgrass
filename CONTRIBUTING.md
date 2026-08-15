# Contributing

* Short rules for changing touchgrass's code.
* Run the checks at the bottom before every commit.

## Contents

* [Agents](#agents)
* [Where Code Goes](#where-code-goes)
* [Errors](#errors)
* [Clones](#clones)
* [Names](#names)
* [Comments](#comments)
* [Tests](#tests)
* [Before You Commit](#before-you-commit)
* [Pull Requests](#pull-requests)

## Agents

* AI help is welcome. Treat what it gives you as a first draft from someone who does not know this project.
* You are the author. You answer for every line, whoever or whatever wrote it.
* Pull requests that break these rules will be closed without review.

**Check the output**

* Read every line before you commit it. If you cannot explain why a line is there, take it out.
* Check it against the rules in this file. Agents often use `unwrap`, add clones, break the module rules, or invent short names.
* Check that every crate, function & setting it uses really exists. Agents make them up.
* Before an agent writes something from scratch, have it check for a crate that already does the job, such as on crates.io or in the crates listed in [ARCHITECTURE.md](ARCHITECTURE.md#dependencies). Use a well kept crate instead of rewriting it.
* Do not trust an agent that says the tests pass. Run the checks below yourself.
* Watch for changes you did not ask for, such as edits to other files, new crates, or removed tests.
* Keep agent changes small, so you can really review them. Split a big one into several commits.

**Commits & pull requests**

* Make every commit yourself, from your own git account, with your own name & email.
* Agents must not commit, push, or open pull requests for you.
* No agent may appear as an author, committer, or `Co-authored-by` line.
* If an agent wrote a large part of a change, say so in the pull request.

## Where Code Goes

* One job per file. See [ARCHITECTURE.md](ARCHITECTURE.md#modules).

| File | Rule |
| --- | --- |
| `main.rs` | Reads the command & calls the right module. The only file that joins modules together. |
| `config.rs` | Loads the tool list. Never touches `data.json`. |
| `tracker.rs` | Counts time. The only module that writes time. |
| `idle.rs` | Says if you're at the computer. Knows nothing about tools or time. |
| `storage.rs` | Loads & saves `data.json`. Never prints. |
| `dashboard.rs` | Adds up time & prints it. Never writes files. |
| `autostart.rs` | Adds & removes the login file. Nothing else. |

* Code that differs between Mac & Linux stays in `idle.rs` & `autostart.rs`. Everything else is shared.
* Save folders come from `dirs`. Never write a path like `~/.config` by hand outside those files.
* If a change needs to break one of these rules, open an issue first.
* Small, focused files. Use `foo.rs` with a `foo/` folder next to it, not `mod.rs`.

## Errors

* Do not use `unwrap` or `expect`, in tests too. Clippy blocks them.
* Use `?` & say what failed, such as `could not read config.toml`.
* When something being missing is normal, use `match` or `if let` on the `Option`.
* Do not invent a default value just to get past an error.
* Never delete `data.json`. If it's broken, rename it. See [ARCHITECTURE.md](ARCHITECTURE.md#errors).

## Clones

* Borrow with `&` when the caller still needs the value.
* Move the value when the caller is done with it.
* Clone only when two owners really need it & say why in a comment.
* Do not hide a copy with `.to_owned()` or `.to_vec()`.

## Names

* Clear, full names, such as `idle_after` or `tool_name`. No short or clever names.
* Put repeated or unclear values in a named `const` with a short `///` comment, not inline.
* Timing values (5 second check, 1 minute save, 2 minute idle) live as constants in `tracker.rs`.
* Heatmap colors & hour levels live as constants in `dashboard.rs`.
* Tests use the same constants. They do not copy raw values.

## Comments

* Every file starts with a `//!` line that says what it is for.
* Every public item has a short `///` comment.
* Comments explain why, in plain words & only where the code is not obvious.
* Every `unsafe` block has a `// SAFETY:` comment.
* No em dashes, in code, comments, docs, or commit messages.

## Tests

* Tests return `Result` & use `?` instead of `unwrap`.
* Keep unit tests at the bottom of the file they check. Tests that run the whole program go in `tests/`.
* `dashboard.rs` must be testable with no real clock. Pass in today's date.
* Matching & priority must be testable with no real programs. Pass in a list of program names.
* Test the edges: midnight, a week across New Year, an empty `data.json`.

## Before You Commit

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets
```

* Keep commits small: one change per commit.
* Start the message with `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, or `chore:`.
* Keep the message short & plain.

## Pull Requests

* One topic per pull request.
* Say what changed & how you checked it. If you ran touchgrass, say which OS & terminal.
* If the output changed, paste what it looks like now.
* Link the issue it fixes, such as `Fixes #12`.
