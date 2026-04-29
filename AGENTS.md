# AGENTS.md

## Local `mcat` Command

This checkout is the only source of truth for the local `mcat` executable.

- Do not copy or symlink `mcat` into `~/.local/bin`, `~/.cargo/bin`, or another PATH directory.
- The user's `~/.zshrc` should expose `mcat` with:

  ```zsh
  alias mcat="/Users/ringcrl/saga/mcat/target/release/mcat"
  ```

- After changing Rust code, run:

  ```sh
  cargo build --release -p mcat
  ```

- A new interactive zsh session will pick up the alias automatically. In an existing shell, run:

  ```sh
  source ~/.zshrc
  ```
