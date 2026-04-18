use std::path::PathBuf;
use sysinfo::{System, RefreshKind};
use serde::{Serialize, Deserialize};
use std::fs;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BenchmarkResult {
    pub command: String,
    pub duration_secs: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exit_status: Option<i32>,
}

pub struct ShellState {
    pub prev_dir: Option<PathBuf>,
    pub sys: System,
    pub history_path: PathBuf,
    pub bench_history_path: PathBuf,
    pub aliases_path: PathBuf,
    pub bench_results: Vec<BenchmarkResult>,
    pub aliases: HashMap<String, String>,
    pub last_exit_status: Option<i32>,
}

impl ShellState {
    pub fn new() -> Self {
        let data_dir = dirs::data_local_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
        let vantage_dir = data_dir.join("vantage");
        if !vantage_dir.exists() {
            let _ = std::fs::create_dir_all(&vantage_dir);
        }
        let history_path = vantage_dir.join("history");
        let bench_history_path = vantage_dir.join("benchmarks.json");
        let aliases_path = vantage_dir.join("aliases.json");

        let bench_results = if let Ok(content) = fs::read_to_string(&bench_history_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        let aliases = if let Ok(content) = fs::read_to_string(&aliases_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self {
            prev_dir: None,
            sys: System::new_with_specifics(RefreshKind::nothing()),
            history_path,
            bench_history_path,
            aliases_path,
            bench_results,
            aliases,
            last_exit_status: Some(0), // Default to success
        }
    }

    pub fn save_benchmarks(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.bench_results) {
            let _ = fs::write(&self.bench_history_path, content);
        }
    }

    pub fn save_aliases(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.aliases) {
            let _ = fs::write(&self.aliases_path, content);
        }
    }

    pub fn add_benchmark(&mut self, command: String, duration_secs: f64, exit_status: Option<i32>) {
        let result = BenchmarkResult {
            command,
            duration_secs,
            timestamp: chrono::Utc::now(),
            exit_status,
        };
        self.bench_results.push(result);
        self.save_benchmarks();
    }
}
