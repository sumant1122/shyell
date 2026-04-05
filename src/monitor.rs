use std::path::Path;
use crate::state::{ShellState, BenchmarkResult};

pub struct Monitor;

impl Monitor {
    pub fn pre_flight_check(state: &mut ShellState) {
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
            let git_info = Self::get_git_info(&cwd);
            return Some(format!("🦀 Rust Project{}", 
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for Node.js project
        if cwd.join("package.json").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!("📦 Node.js Project{}", 
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for Python project
        if cwd.join("requirements.txt").exists() || cwd.join("pyproject.toml").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!("🐍 Python Project{}", 
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for Git (Generic)
        if let Some(git_info) = Self::get_git_info(&cwd) {
            return Some(format!("📜 {}", git_info));
        }

        None
    }

    fn get_git_info(path: &Path) -> Option<String> {
        let dot_git = path.join(".git");
        if !dot_git.exists() { return None; }

        let head_path = dot_git.join("HEAD");
        if let Ok(head_content) = std::fs::read_to_string(head_path) {
            if head_content.starts_with("ref: refs/heads/") {
                let branch = head_content.trim_start_matches("ref: refs/heads/").trim();
                return Some(format!(" {}", branch));
            } else if !head_content.trim().is_empty() {
                return Some(format!(" ({:.7})", head_content.trim()));
            }
        }
        Some("📜 Git Repo".to_string())
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

