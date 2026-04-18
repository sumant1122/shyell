# Vantage Documentation

This document provides a detailed overview of the Vantage shell, its features, commands, and internal architecture.

## ✨ Detailed Features

### 🚦 Real-time Monitoring
Vantage keeps you informed about your system's health without needing to open a separate monitor.
- **CPU Usage**: Real-time global CPU load percentage.
- **Memory Usage**: Current system memory consumption.
- **Visual Alerts**: Stats are color-coded (Green/Yellow/Red) based on load thresholds.

### 🦀 Semantic Context
The shell is aware of your environment. When you navigate into a project directory, Vantage detects the stack and displays relevant metadata:
- **Rust**: 🦀 Rust Project (plus Git branch)
- **Node.js**: 📦 Node.js Project (plus Git branch)
- **Python**: 🐍 Python Project (plus Git branch)
- **Go**: 🐹 Go Project
- **PHP**: 🐘 PHP Project
- **Java**: ☕ Java Project
- **Ruby**: 💎 Ruby Project
- **Git**:  branch_name or commit hash

### ⏱️ Flight Recorder (Benchmarking)
Prefix any command with `bench` to track its performance.
- **Persistence**: Results are saved to `~/.local/share/vantage/benchmarks.json`.
- **Regression Alerts**: Vantage compares the current run against the historical average.
- **Visual Trends**: Displays a sparkline (e.g., ` ▂▃▄▅`) showing the performance trend of the last 15 runs.

### ⌨️ UI/UX Enhancements
- **Syntax Highlighting**: Built-in commands are highlighted as you type.
- **Inline Hints**: Grayed-out suggestions appear for commands based on your history and built-ins.
- **Tab Completion**: Intelligent completion for paths, binaries, and internal commands.

---

## ⌨️ Command Reference

### Built-in Commands

| Command | Description |
| :--- | :--- |
| `sys` | Displays a detailed hardware/OS report (CPU, Mem, Disk, Uptime). |
| `top` | Lists the top 10 CPU-consuming processes. |
| `bench <cmd>` | Executes a command and logs its performance. |
| `history` | Displays the performance history of benchmarked commands. |
| `alias [name='cmd']`| Defines or lists command aliases. Persistent across sessions. |
| `unalias <name>` | Removes a previously defined alias. |
| `export KEY=VAL` | Sets an environment variable for the current session. |
| `cd [dir]` | Changes the current directory (supports `cd -`). |
| `pwd` | Prints the current working directory. |
| `echo [args]` | Prints arguments to stdout. |
| `help` | Displays a quick-start guide. |
| `exit` | Gracefully exits the shell. |

### Advanced Syntax

Vantage supports standard POSIX-like operators for control flow:
- **Pipes (`|`)**: `ls | grep .rs`
- **Redirection (`>`, `>>`, `<`)**: `sys > report.txt`
- **Logical AND (`&&`)**: `cargo build && ./target/debug/app` (stops on failure)
- **Logical OR (`||`)**: `test -f file || echo "Missing"` (stops on success)
- **Sequence (`;`)**: `echo "Start" ; sleep 2 ; echo "Done"`

---

## 🛠 Project Architecture

Vantage is modularly designed for performance and extensibility:

- **`src/main.rs`**: The entry point. Handles the REPL loop, prompt rendering, and integration with `rustyline`.
- **`src/parser.rs`**: A custom tokenizer and parser. Handles alias expansion and converts raw input into a sequence of `PipelineExecution` tasks.
- **`src/executor.rs`**: The execution engine. Manages process spawning, stdio piping, and logical operator short-circuiting.
- **`src/builtins.rs`**: Implementation of all internal shell commands.
- **`src/monitor.rs`**: The "Semantic Context" engine. Polls system stats and identifies project structures.
- **`src/state.rs`**: Manages persistence for history, aliases, and benchmarks using XDG-compliant paths.
- **`src/completion.rs`**: Custom `rustyline` helper that provides completion, hinting, and highlighting.
