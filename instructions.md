Here is the complete guide on how to initialize and build the project, written in Markdown and in English, ready to be included in your repository or a separate `BUILD.md` file.

---

# 🛠 Setting Up and Building the Rust Project

This guide explains how to set up the Rust environment, initialize the project from scratch, and compile a static, optimized binary for the ARMv7 architecture (SDX55).

## 1. Project Initialization
First, create the project structure using Cargo, the Rust package manager. Open your terminal and run:

```bash
# Create the project folder and structure
cargo new atcli-rs
cd atcli-rs
```

This command creates a folder named `atcli-rs` with a `src/` directory and a `Cargo.toml` file.

## 2. Configuring `Cargo.toml`
Replace the entire content of the `Cargo.toml` file with the following configuration. This includes "aggressive" optimization flags to minimize the final binary size (crucial for embedded systems).

```toml
[package]
name = "atcli"
version = "0.1.0"
edition = "2021"
description = "Safe and lightweight AT Command CLI for SDX55 Modems"

[dependencies]
# No external dependencies are used to keep the binary as small as possible.

[profile.release]
opt-level = "z"     # Optimize specifically for binary size
lto = true          # Enable Link Time Optimization (removes unused code)
codegen-units = 1   # Compile as a single unit for maximum compression
panic = "abort"     # Remove panic unwinding overhead
strip = true        # Automatically strip debug symbols from the binary
```

## 3. Adding the Source Code
Open `src/main.rs` and replace its contents with the final, commented Rust code provided in the previous step (the version with the `TERMINATORS` array and `BufReader` logic).

## 4. Installing the ARM Target
To compile for the SDX55 modem (ARMv7 architecture), you must install the cross-compilation target:

```bash
rustup target add armv7-unknown-linux-gnueabihf
```

## 5. Compiling the Static Binary
To ensure the binary runs on the router regardless of the available system libraries (like `glibc`), we must force a **static build**. We do this by passing a specific flag to the compiler via `RUSTFLAGS`.

Run the following command:

```bash
RUSTFLAGS="-C target-feature=+crt-static" cargo build --target armv7-unknown-linux-gnueabihf --release
```

**Where is the binary?**
Once the process finishes, your executable will be located at:
`target/armv7-unknown-linux-gnueabihf/release/atcli`

## 6. (Optional) Extreme Compression with UPX
If you need the smallest possible file (ideal for loading into a router's RAM), use the `upx` tool to compress the binary:

```bash
# This will usually reduce the size from ~600KB down to ~150-200KB
upx --best --lzma target/armv7-unknown-linux-gnueabihf/release/atcli
```

---

### Summary of Files in the Repo
Your final GitHub repository should look like this:
* `README.md`: General overview.
* `reversing_readme.md`: Technical details of the reverse engineering.
* `Cargo.toml`: Build configuration.
* `src/main.rs`: The Rust source code.
