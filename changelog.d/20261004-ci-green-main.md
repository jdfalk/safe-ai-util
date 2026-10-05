### Fixed

#### CI on main is green again

Five separate faults kept the main branch's CI red.

- **Clippy:** the 1.97 toolchain flags collapsible `if`s in `git.rs` and
  `editor.rs` and a `useless_vec` in `security/mod.rs` test code. Under
  `-D warnings` these fail the build jobs.
- **Formatting:** `cargo fmt --check` failed in 14 source files. These files
  are now formatted, and nothing else changed in them.
- **aarch64 build:** the `aarch64-unknown-linux-gnu` release build linked with
  the host x86_64 `cc`. That handed `--fix-cortex-a53-843419` to an x86_64 lld,
  which rejects it. `.cargo/config.toml` now selects `aarch64-linux-gnu-gcc`.
- **Super-linter:** the `*_CONFIG_FILE` values in `super-linter-ci.env` were
  paths that included `.github/linters/`. Super-linter resolves them relative
  to that same directory, so it stopped with "rules file doesn't exist" before
  linting anything.
- **Docker security job:** this job had three faults, and one fix covers each.
  - It referenced the image as `:<commit sha>`, a tag that is never pushed.
    It now uses the primary build's digest.
  - It lacked the `id-token: write` permission that `attest-sbom` needs.
  - It hid both failures behind `|| echo`.

  Fixing it also exposed an image bug. The runtime `WORKDIR` was owned by
  root, so every invocation, including the default `--help`, failed when the
  binary tried to create `./logs`.
