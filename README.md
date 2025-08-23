# cargo-builder

A Cargo build wrapper that provides errors-only output with optional logging.

## 🚀 Features

- **Errors-only output**: Suppresses warnings by default, shows only compilation errors
- **Smart logging**: Creates error log files only when errors occur, auto-cleans on success  
- **Pass-through args**: Fully supports all `cargo build` arguments and flags
- **Color control**: Configurable color output for both terminal and log files
- **Cross-platform**: Works on Windows, macOS, and Linux
- **Zero overhead**: Minimal performance impact over plain `cargo build`

## 📦 Installation

### Option 1: Install from Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/kittynXR/cargo-builder.git
cd cargo-builder

# Build and install
cargo install --path .
```

This installs `cargo-builder` to your Cargo bin directory (usually `~/.cargo/bin`).

### Option 2: Manual Installation

```bash
# Build the release binary
cargo build --release

# Copy to a directory in your PATH
# Windows:
copy target\release\cargo-builder.exe C:\Users\%USERNAME%\.cargo\bin\

# macOS/Linux:
cp target/release/cargo-builder ~/.cargo/bin/
```

### Option 3: Local Development

If you just want to try it out without installing:

```bash
cargo build --release
# Use the binary directly:
./target/release/cargo-builder  # Unix
# or
target\release\cargo-builder.exe  # Windows
```

## ✅ Verification

After installation, verify it works:

```bash
# Check installation (preferred Cargo subcommand style)
cargo builder --help

# Test on any Rust project
cd your-rust-project
cargo builder

# Also works as direct binary call
cargo-builder --help
```

## 🛠️ Usage

### Basic Usage

```bash
# Basic usage - suppress warnings, show compilation progress and errors
cargo builder

# Pass cargo build flags directly (no -- separator needed!)
cargo builder --release --workspace

# Include warnings when you need them  
cargo builder --include-warnings

# Quiet mode for scripts
cargo builder --quiet
```

### Alternative: Direct Binary Call

```bash
# Also works as direct binary call
cargo-builder --release --workspace
cargo-builder --include-warnings

# Legacy syntax still supported for compatibility
cargo builder -- --release --workspace
```

### Advanced Usage

```bash
# Custom log file location
cargo builder --log ./my-build-errors.log --release

# Keep log file even on successful builds (useful for CI)
cargo builder --log-on-success --workspace

# Show all cargo output (including warnings, debug output)
cargo builder --show-build-output --verbose

# Disable colors in terminal output
cargo builder --terminal-color never --release

# Enable colors in log files  
cargo builder --log-color always --workspace

# Mix tool flags with cargo flags seamlessly
cargo builder --quiet --include-warnings --release -p mypackage
```

### Real-World Examples

```bash
# Development: Focus on errors, ignore warnings, see build progress
cargo builder

# Pre-commit: Check with warnings included
cargo builder --include-warnings --workspace

# CI/Release: Build optimized with persistent logging
cargo builder --log-on-success --release

# Debug build issues: See all cargo output
cargo builder --show-build-output --include-warnings

# Cross-compilation
cargo builder --target x86_64-pc-windows-gnu

# Workspace builds with specific packages
cargo builder --workspace --exclude problematic-crate

# Complex multi-flag example
cargo builder --quiet --include-warnings --log-on-success --release -p core -p utils
```

## Command Line Options

- `--log <PATH>`: Target log file path (default: `<workspace>/target/build-errors.log`)
- `--log-on-success`: Keep the log file even on successful builds
- `--log-color <auto|never|always>`: Color control for log file (default: never)
- `--terminal-color <auto|never|always>`: Color control for terminal output
- `--include-warnings`: Do not suppress rustc warnings
- `--show-build-output`: Also mirror Cargo's raw stderr output
- `-q, --quiet`: Minimize plugin output messages

## How It Works

1. **Smart Argument Parsing**: Separates tool flags from cargo flags automatically - no `--` separator required
2. **Warning Suppression**: Adds `-Awarnings` to `RUSTFLAGS` unless `--include-warnings` is specified  
3. **Hybrid Output**: Shows compilation progress via stderr, parses errors/warnings via JSON stdout
4. **Error Extraction**: Uses `--message-format=json-diagnostic-rendered-ansi` for precise error handling
5. **Smart Logging**: Creates log files only when errors occur, removes them on successful builds
6. **Exit Code Preservation**: Returns the same exit code as the underlying `cargo build` command

### What You See vs What You Don't

✅ **Always Visible:**
- Compilation progress ("Compiling package v1.0.0...")
- Build status and timing
- Error messages with full formatting

❌ **Suppressed by Default:**
- Warning messages (enable with `--include-warnings`)
- Verbose cargo output (enable with `--show-build-output`)

✅ **Smart Features:**
- Pass cargo flags directly: `cargo builder --release --workspace`  
- Mix tool and cargo flags: `cargo builder --quiet --release -p mypackage`
- Automatic log cleanup on successful builds

## 🔍 Why cargo-builder?

### Before: Noisy `cargo build`
```
$ cargo build
   Compiling my-project v0.1.0 (/path/to/project)
warning: unused variable: `x`
 --> src/main.rs:2:9
  |
2 |     let x = 42;
  |         ^ help: if this is intentional, prefix it with an underscore: `_x`

warning: function `helper` is never used
 --> src/lib.rs:10:4
   |
10 | fn helper() -> i32 {
   |    ^^^^^^

error[E0425]: cannot find function `undefined_func` in this scope
 --> src/main.rs:5:25
  |
5 |     let result = undefined_func();
  |                  ^^^^^^^^^^^^^^ not found in this scope

error: could not compile `my-project` due to 1 previous error; 2 warnings emitted
```

### After: Clean `cargo builder`
```
$ cargo builder
cargo-builder: Running build with errors-only output...
cargo-builder: Starting build...
error[E0425]: cannot find function `undefined_func` in this scope
 --> src/main.rs:5:25
  |
5 |     let result = undefined_func();
  |                  ^^^^^^^^^^^^^^ not found in this scope

cargo-builder: Build failed with errors
cargo-builder: Error details written to: target/build-errors.log
```

## 📋 Examples

### Error Handling & Logging

When builds fail, errors are:
1. **Displayed cleanly** in the terminal (warnings suppressed)
2. **Logged persistently** to `target/build-errors.log` 
3. **Preserved with formatting** for easy review

The log file contains:
```
cargo-builder error log
======================

error[E0425]: cannot find function `undefined_func` in this scope
 --> src/main.rs:5:25
  |
5 |     let result = undefined_func();
  |                  ^^^^^^^^^^^^^^ not found in this scope
```

### Successful Builds

On success, the log file is automatically removed (unless `--log-on-success` is used):

## Architecture

- `src/main.rs`: CLI argument parsing and orchestration
- `src/runner.rs`: Cargo process spawning and environment setup
- `src/diagnostics.rs`: JSON message parsing and formatting
- `src/logging.rs`: Error log file management
- `src/term.rs`: Terminal and color detection
- `src/util.rs`: Workspace metadata helpers

## License

MIT OR Apache-2.0