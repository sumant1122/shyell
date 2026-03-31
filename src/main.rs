mod state;
mod parser;
mod executor;
mod builtins;
mod monitor;

use std::env;
use std::path::PathBuf;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind};

use crate::state::ShellState;
use crate::parser::parse_commands;
use crate::executor::execute_commands;
use crate::monitor::Monitor;

fn get_prompt(state: &mut ShellState) -> String {
    // Refresh system stats for the prompt
    state.sys.refresh_specifics(
        RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
    );

    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("?"));
    let mut cwd_str = cwd.to_string_lossy().to_string();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        if cwd_str.starts_with(&home_str) {
            cwd_str = cwd_str.replacen(&home_str, "~", 1);
        }
    }
    
    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    
    // Observed Environment: Active Dashboard stats
    let cpu_usage = state.sys.global_cpu_usage();
    let mem_pct = (state.sys.used_memory() as f64 / state.sys.total_memory() as f64) * 100.0;
    
    let cpu_color = if cpu_usage > 80.0 { "1;31" } else if cpu_usage > 50.0 { "1;33" } else { "1;32" };
    let mem_color = if mem_pct > 80.0 { "1;31" } else if mem_pct > 50.0 { "1;33" } else { "1;32" };

    let mut prompt = String::new();
    
    // Add Semantic Context if available
    if let Some(ctx) = Monitor::get_semantic_context() {
        prompt.push_str(&format!("\x1b[38;5;244m{}\x1b[0m\n", ctx));
    }

    // Dashboard readout
    prompt.push_str(&format!(
        "\x1b[{cpu_color}mCPU:{:.0}%\x1b[0m \x1b[{mem_color}mMEM:{:.0}%\x1b[0m ",
        cpu_usage, mem_pct
    ));

    // Main prompt
    prompt.push_str(&format!(
        "\x1b[1;32m{}\x1b[0m:\x1b[1;34m{}\x1b[0m\x1b[1;37m$ \x1b[0m",
        user, cwd_str
    ));

    prompt
}

fn main() {
    let mut state = ShellState::new();
    let mut rl = DefaultEditor::new().unwrap();

    let _ = rl.load_history(&state.history_path);

    loop {
        let readline = rl.readline(&get_prompt(&mut state));
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);
                
                match shell_words::split(line) {
                    Ok(words) => {
                        let cmds = parse_commands(words);
                        execute_commands(cmds, &mut state);
                    }
                    Err(e) => eprintln!("Parse error: {}", e),
                }
            },
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => {
                println!("exit");
                break;
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    let _ = rl.save_history(&state.history_path);
    state.save_benchmarks();
}
