# 🐚 Vantage - The Observed Environment Shell

Vantage is a performance-focused shell written in Rust, designed to be more than just a command runner. It acts as an **Observed Environment**, providing real-time system health, historical performance insights, and proactive guardrails.

## 🚀 The "Observed Environment" Philosophy

Unlike traditional shells that are passive, Vantage is active. It monitors your system resources *while* you work and provides context that helps you make better decisions.

### Key Differentiators:
- **Active Dashboard Prompt**: Real-time CPU and Memory status integrated directly into your prompt.
- **Status-Aware Symbol**: A modern `❯` symbol that turns **green** on success and **red** on failure.
- **Semantic Context**: Automatically detects if you are in a Rust, Node.js, or Git project and shows relevant system tags (e.g., **Git branch** or project type).
- **Standards Compliant**: Uses XDG base directories for config/history (e.g., `~/.local/state/vantage/`) to keep your `$HOME` clean and accurately manages standard shell variables like `PWD`/`OLDPWD`.
- **Advanced Native Tokenizer**: Employs a custom, ultra-fast built-in lexer that strictly respects variable expansion rules inside single and double quotes, with `$PATH` caching for instant `<TAB>` completions.
- **Pre-flight Guardrails**: Warns you if system resources (CPU/RAM) are too high before you execute a command.
- **Flight Recorder (Benchmarking)**: Automatically stores benchmarking results in a JSON database and alerts you to performance regressions.
- **Surgical Built-ins**: Native, high-performance implementations of `sys`, `top`, and `history`.

## 🛠 Features

- **Pipelines & Redirections**: Full support for `|`, `>`, `>>`, and `<`.
- **Intelligent Tab Completion**: Autocomplete built-in commands, external binaries in your `$PATH`, and file paths.
- **Variable Expansion**: Supports standard `$VAR` and `${VAR}` syntax.
- **Benchmark Command**: Prefix any command with `bench` to measure its impact.
- **History Management**: Persistent command history via `rustyline`.

## 📂 Project Structure

- `src/main.rs`: Entry point and dynamic prompt logic.
- `src/state.rs`: Centralized shell state and benchmarking history.
- `src/monitor.rs`: The "Observer" engine (resource checks, semantic context).
- `src/executor.rs`: Robust pipeline and command execution.
- `src/parser.rs`: Shell command parsing and expansion.
- `src/builtins.rs`: Native shell commands.

## 🤝 Community and Contributing

We welcome contributions of all kinds! Please see our:
- [Contributing Guidelines](CONTRIBUTING.md) for how to get started.
- [Code of Conduct](CODE_OF_CONDUCT.md) for our community standards.
- [Changelog](CHANGELOG.md) for a history of changes.

## 📄 License

This project is licensed under the terms of the [LICENSE](LICENSE) file.

## 📥 Installation

```bash
cargo build --release
./target/release/Vantage
```

## ⌨️ Built-in Commands

- `sys`: Comprehensive system overview (OS, Kernel, CPU, Mem, Disk).
- `top`: Top 10 processes by CPU usage.
- `history`: View historical benchmark results.
- `bench <cmd>`: Execute and benchmark a command.
- `cd`, `pwd`, `echo`, `exit`: Standard shell operations.

## 📖 Example Usage

### 1. System Health Check
Get a quick dashboard of your machine's current state:
```bash
sys
```

### 2. Performance Benchmarking
Measure exactly how long a build or script takes:
```bash
bench cargo build --release
```
*Vantage will automatically record this and alert you if future runs are significantly slower.*

### 3. Advanced Pipelines & Redirection
Combine tools and save output just like in Bash:
```bash
ls -la | grep "rs" > rust_files.txt
```

### 4. Project-Aware Context
Enter a directory and watch the prompt adapt:
```bash
cd my_rust_project
# The prompt will instantly highlight that you are inside a Rust context and display your active Git branch.
```

### 5. Historical Analysis
Review your past performance benchmarks:
```bash
history
```
