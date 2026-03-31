use std::path::{Path, PathBuf};
use crate::state::{ShellState, BenchmarkResult};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind};
use walkdir::WalkDir;

pub struct Monitor;

impl Monitor {
    pub fn pre_flight_check(state: &mut ShellState) {
        state.sys.refresh_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
        );

        let mem_used_pct = (state.sys.used_memory() as f64 / state.sys.total_memory() as f64) * 100.0;
        let cpu_usage = state.sys.global_cpu_usage();

        if mem_used_pct > 90.0 {
            println!("\x1b[1;33m[!] High Memory Usage: {:.1}% - System may be sluggish.\x1b[0m", mem_used_pct);
        }
        if cpu_usage > 90.0 {
            println!("\x1b[1;33m[!] High CPU Load: {:.1}% - Pre-flight warning.\x1b[0m", cpu_usage);
        }
    }

    pub fn get_semantic_context() -> Option<String> {
        let cwd = std::env::current_dir().ok()?;
        
        // Check for Rust project
        if cwd.join("Cargo.toml").exists() {
            let target_size = Self::get_dir_size(&cwd.join("target"));
            return Some(format!("🦀 Rust Project | target: {}", Self::format_size(target_size)));
        }

        // Check for Node.js project
        if cwd.join("package.json").exists() {
            let nm_size = Self::get_dir_size(&cwd.join("node_modules"));
            return Some(format!("📦 Node.js Project | node_modules: {}", Self::format_size(nm_size)));
        }

        // Check for Python project
        if cwd.join("requirements.txt").exists() || cwd.join("pyproject.toml").exists() {
            return Some("🐍 Python Project".to_string());
        }

        // Check for Git
        if cwd.join(".git").exists() {
            return Some("📜 Git Repository".to_string());
        }

        None
    }

    fn get_dir_size(path: &Path) -> u64 {
        if !path.exists() { return 0; }
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum()
    }

    fn format_size(bytes: u64) -> String {
        if bytes == 0 { return "0B".into(); }
        let units = ["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_idx = 0;
        while size >= 1024.0 && unit_idx < units.len() - 1 {
            size /= 1024.0;
            unit_idx += 1;
        }
        format!("{:.1}{}", size, units[unit_idx])
    }

    pub fn check_regression(state: &ShellState, command: &str, current_duration: f64) {
        let history: Vec<&BenchmarkResult> = state.bench_results.iter()
            .filter(|r| r.command == command)
            .collect();

        if history.len() >= 3 {
            let avg_duration: f64 = history.iter().map(|r| r.duration_secs).sum::<f64>() / history.len() as f64;
            if current_duration > avg_duration * 1.3 {
                println!("\x1b[1;35m[!] Performance Regression Detected!\x1b[0m");
                println!("    Current: {:.2}s | Average: {:.2}s (+{:.0}%)", 
                    current_duration, avg_duration, (current_duration / avg_duration - 1.0) * 100.0);
            }
        }
    }
}
