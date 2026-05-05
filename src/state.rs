use crate::builtins::{get_builtins, BuiltinCommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use sysinfo::{RefreshKind, System};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BenchmarkResult {
    pub command: String,
    pub duration_secs: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exit_status: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShellConfig {
    pub prompt_color_user: String,
    pub prompt_color_cwd: String,
    pub default_aliases: HashMap<String, String>,
    pub initial_env: HashMap<String, String>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        let mut default_aliases = HashMap::new();
        default_aliases.insert("ll".to_string(), "ls -la".to_string());
        
        Self {
            prompt_color_user: "1;32".to_string(),
            prompt_color_cwd: "1;34".to_string(),
            default_aliases,
            initial_env: HashMap::new(),
        }
    }
}

pub struct ShellState {
    pub prev_dir: Option<PathBuf>,
    pub sys: System,
    pub history_path: PathBuf,
    pub bench_history_path: PathBuf,
    pub aliases_path: PathBuf,
    pub config_path: PathBuf,
    pub bench_results: Vec<BenchmarkResult>,
    pub aliases: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
    pub last_exit_status: Option<i32>,
    pub builtins: HashMap<String, Box<dyn BuiltinCommand>>,
    pub config: ShellConfig,
}

impl ShellState {
    pub fn new() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")));
        let shyell_dir = data_dir.join("shyell");
        if !shyell_dir.exists() {
            let _ = std::fs::create_dir_all(&shyell_dir);
        }
        let history_path = shyell_dir.join("history");
        let bench_history_path = shyell_dir.join("benchmarks.json");
        let aliases_path = shyell_dir.join("aliases.json");
        let config_path = shyell_dir.join("config.json");

        let bench_results = if let Ok(content) = fs::read_to_string(&bench_history_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut aliases = if let Ok(content) = fs::read_to_string(&aliases_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        let config = if let Ok(content) = fs::read_to_string(&config_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            let default_config = ShellConfig::default();
            if let Ok(content) = serde_json::to_string_pretty(&default_config) {
                let _ = fs::write(&config_path, content);
            }
            default_config
        };

        // Merge default aliases from config if not already set
        for (k, v) in &config.default_aliases {
            aliases.entry(k.clone()).or_insert_with(|| v.clone());
        }

        let mut env_vars: HashMap<String, String> = std::env::vars().collect();
        // Overlay initial env from config
        for (k, v) in &config.initial_env {
            env_vars.insert(k.clone(), v.clone());
        }

        Self {
            prev_dir: None,
            sys: System::new_with_specifics(RefreshKind::nothing()),
            history_path,
            bench_history_path,
            aliases_path,
            config_path,
            bench_results,
            aliases,
            env_vars,
            last_exit_status: Some(0),
            builtins: get_builtins(),
            config,
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

    pub fn set_env(&mut self, key: &str, val: String) {
        self.env_vars.insert(key.to_string(), val.clone());
        // We still set it in the current process for library calls, but we also track it for children
        unsafe {
            std::env::set_var(key, val);
        }
    }
}

