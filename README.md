# certificate-chain-rs

This project uses the Rust `openssl` crate and, on Windows, requires a local OpenSSL installation plus the correct environment variables pointing to headers and libraries.

## 1. Install OpenSSL on Windows

Open a PowerShell terminal and install OpenSSL with Chocolatey:

```powershell
choco install openssl -y
```

Typical installation path:

```text
C:\Program Files\OpenSSL-Win64
```

## 2. Configure the current terminal

For the current PowerShell session:

```powershell
$env:OPENSSL_DIR="C:\Program Files\OpenSSL-Win64"
$env:OPENSSL_INCLUDE_DIR="C:\Program Files\OpenSSL-Win64\include"
$env:OPENSSL_LIB_DIR="C:\Program Files\OpenSSL-Win64\lib\VC\x64\MD"
$env:PATH="C:\Program Files\OpenSSL-Win64\bin;$env:PATH"
```

If `lib\VC\x64\MD` does not exist in your installation, check `lib\VC\x64\MT` or `lib` and adjust `OPENSSL_LIB_DIR` accordingly.

## 3. Build

From the project root:

```powershell
cargo build --release
```

Or, for a faster validation:

```powershell
cargo check
```

## Run the program

The executable expects two command-line arguments:

```powershell
cargo run -- www.google.it 443
```

Or, for help:

```powershell
cargo run -- --help
```

## Troubleshooting

- If you get `Could not find directory of OpenSSL installation`, verify `OPENSSL_DIR`, `OPENSSL_INCLUDE_DIR`, and `OPENSSL_LIB_DIR`.
- If you get `could not find native static library 'libssl'`, your `OPENSSL_LIB_DIR` probably points to the wrong subdirectory.
- The GitHub Actions Windows job installs OpenSSL automatically with Chocolatey and sets these variables before running `cargo build --release`.
