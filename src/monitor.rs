use crate::state::{BenchmarkResult, ShellState};
use std::path::Path;

pub struct Monitor;

impl Monitor {
    pub fn pre_flight_check(state: &mut ShellState) {
        let mem_used_pct =
            (state.sys.used_memory() as f64 / state.sys.total_memory() as f64) * 100.0;
        let cpu_usage = state.sys.global_cpu_usage();

        if mem_used_pct > 90.0 {
            println!(
                "\x1b[1;33m[!] High Memory Usage: {:.1}% - System may be sluggish.\x1b[0m",
                mem_used_pct
            );
        }
        if cpu_usage > 90.0 {
            println!(
                "\x1b[1;33m[!] High CPU Load: {:.1}% - Pre-flight warning.\x1b[0m",
                cpu_usage
            );
        }
    }

    pub fn get_semantic_context() -> Option<String> {
        let cwd = std::env::current_dir().ok()?;

        // Check for Rust project
        if cwd.join("Cargo.toml").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!(
                "🦀 Rust Project{}",
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for Node.js project
        if cwd.join("package.json").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!(
                "📦 Node.js Project{}",
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for Python project
        if cwd.join("requirements.txt").exists() || cwd.join("pyproject.toml").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!(
                "🐍 Python Project{}",
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for Go project
        if cwd.join("go.mod").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!(
                "🐹 Go Project{}",
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for PHP project
        if cwd.join("composer.json").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!(
                "🐘 PHP Project{}",
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for Java project
        if cwd.join("pom.xml").exists() || cwd.join("build.gradle").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!(
                "☕ Java Project{}",
                git_info.map(|s| format!(" | {}", s)).unwrap_or_default()
            ));
        }

        // Check for Ruby project
        if cwd.join("Gemfile").exists() {
            let git_info = Self::get_git_info(&cwd);
            return Some(format!(
                "💎 Ruby Project{}",
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
        if !dot_git.exists() {
            return None;
        }

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
        let history: Vec<&BenchmarkResult> = state
            .bench_results
            .iter()
            .filter(|r| r.command == command)
            .collect();

        if history.len() >= 3 {
            let avg_duration: f64 =
                history.iter().map(|r| r.duration_secs).sum::<f64>() / history.len() as f64;

            let mut durations: Vec<f64> = history.iter().map(|r| r.duration_secs).collect();
            durations.push(current_duration);

            let max_d = durations.iter().copied().fold(0.0_f64, f64::max);
            let min_d = durations.iter().copied().fold(f64::INFINITY, f64::min);
            let diff = if max_d - min_d == 0.0 {
                1.0
            } else {
                max_d - min_d
            };
            let sparks = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
            let mut sparkline = String::new();

            let start_idx = if durations.len() > 15 {
                durations.len() - 15
            } else {
                0
            };
            for &d in &durations[start_idx..] {
                let norm = (d - min_d) / diff;
                let idx = (norm * 7.0).round() as usize;
                let color = if d > avg_duration * 1.3 {
                    "\x1b[31m"
                } else if d < avg_duration * 0.7 {
                    "\x1b[32m"
                } else {
                    "\x1b[36m"
                };
                sparkline.push_str(&format!("{}{}\x1b[0m", color, sparks[idx.min(7)]));
            }

            if current_duration > avg_duration * 1.3 {
                println!("\x1b[1;31m[!] Performance Regression Detected!\x1b[0m");
                println!(
                    "    Current: {:.2}s | Average: {:.2}s (+{:.0}%)",
                    current_duration,
                    avg_duration,
                    (current_duration / avg_duration - 1.0) * 100.0
                );
            } else if current_duration < avg_duration * 0.7 {
                println!("\x1b[1;32m[✓] Performance Improvement!\x1b[0m");
                println!(
                    "    Current: {:.2}s | Average: {:.2}s (-{:.0}%)",
                    current_duration,
                    avg_duration,
                    (1.0 - current_duration / avg_duration) * 100.0
                );
            } else {
                println!("\x1b[1;36m[i] Performance is Stable\x1b[0m");
                println!(
                    "    Current: {:.2}s | Average: {:.2}s",
                    current_duration, avg_duration
                );
            }
            println!(
                "    Trend: {} (last {} runs)",
                sparkline,
                durations.len() - start_idx
            );
        }
    }
}
