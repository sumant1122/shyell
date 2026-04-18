<div align="center">

# 🐚 Vantage 

**The Performance-Focused, Environment-Aware Shell**

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)
![Build](https://img.shields.io/badge/Build-Passing-brightgreen.svg)

<br>

Vantage is a modern shell written natively in Rust, designed for developers who care about system health and workflow performance. It proactively monitors system load and benchmarks your commands in real-time.

</div>

---

### ✨ Key Features

- 🚦 **Real-time Monitoring**: Memory and CPU status are tracked directly within your prompt environment.
- 🦀 **Semantic Context**: Automatically detects project types (Rust, Node.js, Python) and Git status to provide relevant context.
- ⚡ **High-Performance Pipeline**: Built with a custom tokenizer for fast command execution and efficient I/O redirection. 
- ⏱️ **Flight Recorder**: Prefix commands with `bench` (e.g., `bench cargo build`) to track execution time and receive alerts on performance regressions. 

### 📥 Getting Started

**Prerequisites:** [Rust 1.80+](https://www.rust-lang.org/tools/install)

```bash
# Clone the repository
git clone https://github.com/your-username/Vantage.git
cd Vantage

# Build and run
cargo run --release
```

### ⌨️ Command Overview

Vantage supports standard POSIX pipelines and redirections alongside its powerful built-ins:

```bash
❯ sys                             # Display a comprehensive system status report
❯ top                             # List top 10 CPU-intensive processes
❯ history                         # View performance history of benchmarked commands
❯ bench ls -la                    # Run a command and measure its performance
❯ ls -la | grep "rs" > output.txt # Robust internal pipeline support
❯ cd my_project                   # Context-aware prompt updates (Git, Language)
```

---

<details>
<summary><b>🛠 Project Architecture</b></summary>
<br>

- `src/main.rs`: Core prompt engine and event loop.
- `src/parser.rs`: Custom lexer/tokenizer with support for pipes and redirection.
- `src/builtins.rs`: Internal shell commands (`sys`, `top`, `bench`, etc.).
- `src/monitor.rs`: System health monitoring and project context detection.
- `src/executor.rs`: Process management and I/O pipeline implementation.
- `src/state.rs`: Persistent state management following XDG standards.

</details>

*For contributing guidelines or code of conduct, please reference our [Contributing](CONTRIBUTING.md) and [License](LICENSE) files.*
