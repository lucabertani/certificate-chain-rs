# certificate-chain-rs

This project uses the Rust `openssl` crate with the `vendored` feature enabled. On Windows, OpenSSL is compiled from source during the Rust build, so you do not need a separate local OpenSSL installation. You do need Perl available in `PATH` because the vendored OpenSSL build uses it during configuration.

## 1. Install Perl on Windows

Open a PowerShell terminal and install Strawberry Perl:

```powershell
winget install --id StrawberryPerl.StrawberryPerl --exact --accept-package-agreements --accept-source-agreements --disable-interactivity
```

After installation, open a new terminal so `perl` is available in `PATH`.

## 2. Build

From the project root:

```powershell
cargo build --release
```

For a faster validation:

```powershell
cargo check
```

On Windows MSVC, the repository already enables static CRT linking through `.cargo/config.toml`, so local `cargo build` uses `-Ctarget-feature=+crt-static` automatically.

## 3. Run the program

The executable expects two command-line arguments:

```powershell
cargo run -- www.google.it 443
```

Or, for help:

```powershell
cargo run -- --help
```

## Troubleshooting

- If you get `Command 'perl' not found`, install Strawberry Perl with the `winget` command above and reopen the terminal.
- If the build still fails right after installing Perl, verify it is available with `perl -v` in a new terminal.
- The GitHub Actions Windows job should install the required prerequisites before running `cargo build --release`.
