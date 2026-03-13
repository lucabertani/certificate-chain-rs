# test-openssl-chain

This project uses the Rust `openssl` crate, which on Windows needs a local OpenSSL installation and the correct environment variables pointing to headers and libraries.

## 1. Install OpenSSL on Windows

Open a PowerShell terminal and install OpenSSL with `winget`:

```powershell
winget install openssl
```

If `winget` is not available or the package is not found, install a 64-bit OpenSSL build manually and note the installation folder.

Typical installation path:

```text
C:\Program Files\OpenSSL-Win64
```

## 2. Verify the installation folders

Before setting environment variables, check that these folders really exist on your machine:

```powershell
Get-ChildItem "C:\Program Files\OpenSSL-Win64"
Get-ChildItem "C:\Program Files\OpenSSL-Win64\include"
Get-ChildItem "C:\Program Files\OpenSSL-Win64\lib"
```

For some OpenSSL distributions, the libraries are under:

```text
C:\Program Files\OpenSSL-Win64\lib\VC\x64\MT
```

That is the layout used by this project.

## 3. Set environment variables for the current terminal

If you only want the configuration for the current PowerShell session, run:

```powershell
$env:OPENSSL_DIR="C:\Program Files\OpenSSL-Win64"
$env:OPENSSL_INCLUDE_DIR="C:\Program Files\OpenSSL-Win64\include"
$env:OPENSSL_LIB_DIR="C:\Program Files\OpenSSL-Win64\lib\VC\x64\MT"
```

These variables mean:

- `OPENSSL_DIR`: root folder of the OpenSSL installation
- `OPENSSL_INCLUDE_DIR`: folder containing the C header files
- `OPENSSL_LIB_DIR`: folder containing the `.lib` files used at build time

## 4. Make the variables permanent

If you want them available in every new terminal, define them as user environment variables.

PowerShell example:

```powershell
[System.Environment]::SetEnvironmentVariable("OPENSSL_DIR", "C:\Program Files\OpenSSL-Win64", "User")
[System.Environment]::SetEnvironmentVariable("OPENSSL_INCLUDE_DIR", "C:\Program Files\OpenSSL-Win64\include", "User")
[System.Environment]::SetEnvironmentVariable("OPENSSL_LIB_DIR", "C:\Program Files\OpenSSL-Win64\lib\VC\x64\MT", "User")
```

After that, close and reopen the terminal or restart VS Code.

If you prefer a GUI tool, you can also create the same variables from Windows System Settings or with PowerToys Environment Variables.

## 5. Validate the configuration

From the project root, run:

```powershell
cargo check
```

If the configuration is correct, the project should compile without `libssl` / `libcrypto` lookup errors.

## 6. Run the program

The executable expects two command-line arguments:

```powershell
cargo run -- www.google.it 443
```

Or, for help:

```powershell
cargo run -- --help
```

## Troubleshooting

- If you get `could not find native static library 'libssl'`, `OPENSSL_LIB_DIR` is probably wrong.
- If you get header-related errors, verify `OPENSSL_INCLUDE_DIR`.
- If you installed a different OpenSSL distribution, the library path may be different from `lib\VC\x64\MT`.
- If `cargo check` works in one terminal but not in another, the variables were probably set only for the current session.
