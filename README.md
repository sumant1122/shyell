# 🐚 Shyell - The Observed Environment Shell

Shyell is a performance-focused shell written in Rust, designed to be more than just a command runner. It acts as an **Observed Environment**, providing real-time system health, historical performance insights, and proactive guardrails.

## 🚀 The "Observed Environment" Philosophy

Unlike traditional shells that are passive, Shyell is active. It monitors your system resources *while* you work and provides context that helps you make better decisions.

### Key Differentiators:
- **Active Dashboard Prompt**: Real-time CPU and Memory status integrated directly into your prompt.
- **Semantic Context**: Automatically detects if you are in a Rust, Node.js, or Git project and shows relevant stats (e.g., `target/` size).
- **Pre-flight Guardrails**: Warns you if system resources (CPU/RAM) are too high before you execute a command.
- **Flight Recorder (Benchmarking)**: Automatically stores benchmarking results in a JSON database and alerts you to performance regressions.
- **Surgical Built-ins**: Native, high-performance implementations of `sys`, `top`, and `history`.

## 🛠 Features

- **Pipelines & Redirections**: Full support for `|`, `>`, `>>`, and `<`.
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

## 📥 Installation

```bash
cargo build --release
./target/release/Shyell
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
*Shyell will automatically record this and alert you if future runs are significantly slower.*

### 3. Advanced Pipelines & Redirection
Combine tools and save output just like in Bash:
```bash
ls -la | grep "rs" > rust_files.txt
```

### 4. Project-Aware Context
Enter a directory and watch the prompt adapt:
```bash
cd my_rust_project
# The prompt will now show the size of the target/ directory
```

### 5. Historical Analysis
Review your past performance benchmarks:
```bash
history
```
