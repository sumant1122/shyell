<div align="center">

# 🐚 Shyell

**The Performance-Focused, Environment-Aware Shell**

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)

<br>

Shyell is a modern shell written natively in Rust, designed for developers who care about system health and workflow performance. It proactively monitors system load and benchmarks your commands in real-time.

</div>

---

### Quick Demo

<img width="948" height="688" alt="shyell" src="https://github.com/user-attachments/assets/a4ce9a23-e2cd-4d4f-9c8f-52cfc54c863a" />


### 📥 Quick Start

**Prerequisites:** [Rust 1.80+](https://www.rust-lang.org/tools/install)

```bash
# Clone the repository
git clone https://github.com/your-username/Shyell.git
cd Shyell

# Build and run
cargo run --release
```

### 📖 Documentation

For a comprehensive guide on features, command usage, and project architecture, please refer to the **[Full Documentation](documentation.md)**.

### ✨ Highlights

- 🚦 **Real-time Monitoring**: Integrated CPU/Memory stats directly in your prompt.
- 🦀 **Semantic Context**: Automatic project stack detection (Rust, Node.js, Go, etc.).
- ⏱️ **Flight Recorder**: Real-time command benchmarking with historical regression alerts and sparklines.
- ⚙️ **Configurable**: Fully customizable via `config.json` (colors, aliases, environment).
- ⚡ **Advanced Execution**: Support for pipes, background jobs (`&`), and complex redirection (`2>`, `&>`).
- 🛡️ **Built for Safety**: Improved signal handling (Ctrl+C) and thread-safe environment management.
- 🧱 **Extensible Architecture**: Trait-based built-in command system for easy expansion.

---

*For contributing guidelines or code of conduct, please reference our [Contributing](CONTRIBUTING.md) and [License](LICENSE) files.*
