use crate::executor::{format_duration, get_output_writer};
use crate::parser::CommandExecution;
use crate::state::ShellState;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use sysinfo::{Disks, ProcessRefreshKind, RefreshKind, System};
use std::collections::HashMap;

pub trait BuiltinCommand: Send + Sync {
    fn name(&self) -> &'static str;
    fn execute(&self, state: &mut ShellState, cmd: &CommandExecution) -> bool;
}

pub struct CdCommand;
impl BuiltinCommand for CdCommand {
    fn name(&self) -> &'static str { "cd" }
    fn execute(&self, state: &mut ShellState, cmd: &CommandExecution) -> bool {
        let target = if cmd.args.len() < 2 {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
        } else if cmd.args[1] == "-" {
            if let Some(prev) = &state.prev_dir {
                prev.clone()
            } else {
                eprintln!("cd: oldpwd not set");
                return true;
            }
        } else {
            PathBuf::from(&cmd.args[1])
        };

        let current = env::current_dir().unwrap_or_default();
        match env::set_current_dir(&target) {
            Ok(_) => {
                state.prev_dir = Some(current.clone());
                state.set_env("OLDPWD", current.to_string_lossy().to_string());
                if let Ok(new_dir) = env::current_dir() {
                    state.set_env("PWD", new_dir.to_string_lossy().to_string());
                }
            }
            Err(e) => eprintln!("cd: {}: {}", target.display(), e),
        }
        true
    }
}

pub struct PwdCommand;
impl BuiltinCommand for PwdCommand {
    fn name(&self) -> &'static str { "pwd" }
    fn execute(&self, _state: &mut ShellState, cmd: &CommandExecution) -> bool {
        let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("shyell: {}", e);
                return true;
            }
        };
        match env::current_dir() {
            Ok(dir) => writeln!(writer, "{}", dir.display()).unwrap_or(()),
            Err(e) => eprintln!("pwd: {}", e),
        }
        true
    }
}

pub struct SysCommand;
impl BuiltinCommand for SysCommand {
    fn name(&self) -> &'static str { "sys" }
    fn execute(&self, state: &mut ShellState, cmd: &CommandExecution) -> bool {
        let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("shyell: {}", e);
                return true;
            }
        };

        writeln!(writer, "\x1b[1;36m--- System Status ---\x1b[0m").unwrap_or(());
        writeln!(writer, "{:<15} {}", "OS:", System::name().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
        writeln!(writer, "{:<15} {}", "Kernel:", System::kernel_version().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
        writeln!(writer, "{:<15} {}", "Hostname:", System::host_name().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
        writeln!(writer, "{:<15} {}", "Uptime:", format_duration(System::uptime())).unwrap_or(());

        let total_mem = state.sys.total_memory() / 1024 / 1024;
        let used_mem = state.sys.used_memory() / 1024 / 1024;
        let mem_pct = if total_mem > 0 { (used_mem as f64 / total_mem as f64 * 100.0) as usize } else { 0 };

        let bar_len = 20;
        let filled = (mem_pct * bar_len) / 100;
        let bar = format!("[{}{}]", "#".repeat(filled), ".".repeat(bar_len - filled));

        writeln!(writer, "{:<15} {} {}% ({}MB / {}MB)", "Memory:", bar, mem_pct, used_mem, total_mem).unwrap_or(());
        writeln!(writer, "{:<15} {:.2}%", "CPU Load:", state.sys.global_cpu_usage()).unwrap_or(());

        let disks = Disks::new_with_refreshed_list();
        if let Some(root) = disks.iter().find(|d| d.mount_point() == std::path::Path::new("/")) {
            let total = root.total_space() / 1024 / 1024 / 1024;
            let avail = root.available_space() / 1024 / 1024 / 1024;
            let used = total - avail;
            let disk_pct = if total > 0 { (used as f64 / total as f64 * 100.0) as usize } else { 0 };
            let filled_disk = (disk_pct * bar_len) / 100;
            let bar_disk = format!("[{}{}]", "#".repeat(filled_disk), ".".repeat(bar_len - filled_disk));
            writeln!(writer, "{:<15} {} {}% ({}GB / {}GB)", "Disk (/):", bar_disk, disk_pct, used, total).unwrap_or(());
        }
        true
    }
}

pub struct TopCommand;
impl BuiltinCommand for TopCommand {
    fn name(&self) -> &'static str { "top" }
    fn execute(&self, state: &mut ShellState, cmd: &CommandExecution) -> bool {
        state.sys.refresh_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
        let mut processes: Vec<_> = state.sys.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

        let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("shyell: {}", e);
                return true;
            }
        };

        writeln!(writer, "\x1b[1;33m{:<8} {:<15} {:<10} {:<10}\x1b[0m", "PID", "Name", "CPU %", "Mem MB").unwrap_or(());
        for p in processes.iter().take(10) {
            writeln!(writer, "{:<8} {:<15} {:<10.2} {:<10}", p.pid(), p.name().to_string_lossy(), p.cpu_usage(), p.memory() / 1024 / 1024).unwrap_or(());
        }
        true
    }
}

pub struct HistoryCommand;
impl BuiltinCommand for HistoryCommand {
    fn name(&self) -> &'static str { "history" }
    fn execute(&self, state: &mut ShellState, cmd: &CommandExecution) -> bool {
        let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("shyell: {}", e);
                return true;
            }
        };
        writeln!(writer, "\x1b[1;35m--- Performance History ---\x1b[0m").unwrap_or(());
        for r in state.bench_results.iter().rev().take(20) {
            writeln!(writer, "[{}] {:<20} | {:.2}s | Exit: {:?}", r.timestamp.format("%Y-%m-%d %H:%M:%S"), r.command, r.duration_secs, r.exit_status).unwrap_or(());
        }
        true
    }
}

pub struct AliasCommand;
impl BuiltinCommand for AliasCommand {
    fn name(&self) -> &'static str { "alias" }
    fn execute(&self, state: &mut ShellState, cmd: &CommandExecution) -> bool {
        let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("shyell: {}", e);
                return true;
            }
        };
        if cmd.args.len() < 2 {
            for (k, v) in &state.aliases {
                writeln!(writer, "alias {}='{}'", k, v).unwrap_or(());
            }
        } else {
            let arg = &cmd.args[1];
            if let Some(idx) = arg.find('=') {
                let key = &arg[..idx];
                let val = &arg[idx + 1..];
                state.aliases.insert(key.to_string(), val.to_string());
                state.save_aliases();
            } else {
                if let Some(v) = state.aliases.get(arg) {
                    writeln!(writer, "alias {}='{}'", arg, v).unwrap_or(());
                } else {
                    eprintln!("alias: {}: not found", arg);
                }
            }
        }
        true
    }
}

pub struct UnaliasCommand;
impl BuiltinCommand for UnaliasCommand {
    fn name(&self) -> &'static str { "unalias" }
    fn execute(&self, state: &mut ShellState, cmd: &CommandExecution) -> bool {
        if cmd.args.len() < 2 {
            eprintln!("unalias: usage: unalias name");
        } else {
            let key = &cmd.args[1];
            if state.aliases.remove(key).is_some() {
                state.save_aliases();
            } else {
                eprintln!("unalias: {}: not found", key);
            }
        }
        true
    }
}

pub struct ExportCommand;
impl BuiltinCommand for ExportCommand {
    fn name(&self) -> &'static str { "export" }
    fn execute(&self, state: &mut ShellState, cmd: &CommandExecution) -> bool {
        let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("shyell: {}", e);
                return true;
            }
        };
        if cmd.args.len() < 2 {
            for (k, v) in &state.env_vars {
                writeln!(writer, "{}={}", k, v).unwrap_or(());
            }
        } else {
            let arg = &cmd.args[1];
            if let Some(idx) = arg.find('=') {
                let key = &arg[..idx];
                let val = &arg[idx + 1..];
                state.set_env(key, val.to_string());
            } else {
                eprintln!("export: usage: export KEY=VALUE");
            }
        }
        true
    }
}

pub struct HelpCommand;
impl BuiltinCommand for HelpCommand {
    fn name(&self) -> &'static str { "help" }
    fn execute(&self, _state: &mut ShellState, cmd: &CommandExecution) -> bool {
        let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("shyell: {}", e);
                return true;
            }
        };
        writeln!(writer, "\x1b[1;32mShyell - Advanced Performance Shell\x1b[0m").unwrap_or(());
        writeln!(writer, "\x1b[1mSystem Performance:\x1b[0m").unwrap_or(());
        writeln!(writer, "  sys         Show system overview (CPU, Mem, Disk, Uptime).").unwrap_or(());
        writeln!(writer, "  top         Show top 10 processes by CPU usage.").unwrap_or(());
        writeln!(writer, "  bench <cmd> Prefix any command to measure its time/resources.").unwrap_or(());
        writeln!(writer, "  history     Show benchmarking history.").unwrap_or(());
        writeln!(writer, "\x1b[1mStandard Commands:\x1b[0m").unwrap_or(());
        writeln!(writer, "  cd [dir]    Change directory ('cd -' for back).").unwrap_or(());
        writeln!(writer, "  pwd         Print current directory.").unwrap_or(());
        writeln!(writer, "  alias       Define or display aliases.").unwrap_or(());
        writeln!(writer, "  unalias     Remove an alias.").unwrap_or(());
        writeln!(writer, "  export      Set environment variables.").unwrap_or(());
        writeln!(writer, "  echo        Print arguments.").unwrap_or(());
        writeln!(writer, "  exit        Exit the shell.").unwrap_or(());
        true
    }
}

pub struct EchoCommand;
impl BuiltinCommand for EchoCommand {
    fn name(&self) -> &'static str { "echo" }
    fn execute(&self, _state: &mut ShellState, cmd: &CommandExecution) -> bool {
        let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("shyell: {}", e);
                return true;
            }
        };
        let output = cmd.args[1..].join(" ");
        writeln!(writer, "{}", output).unwrap_or(());
        true
    }
}

pub struct ExitCommand;
impl BuiltinCommand for ExitCommand {
    fn name(&self) -> &'static str { "exit" }
    fn execute(&self, _state: &mut ShellState, _cmd: &CommandExecution) -> bool {
        std::process::exit(0);
    }
}

pub fn get_builtins() -> HashMap<String, Box<dyn BuiltinCommand>> {
    let mut builtins: HashMap<String, Box<dyn BuiltinCommand>> = HashMap::new();
    let cmds: Vec<Box<dyn BuiltinCommand>> = vec![
        Box::new(CdCommand),
        Box::new(PwdCommand),
        Box::new(SysCommand),
        Box::new(TopCommand),
        Box::new(HistoryCommand),
        Box::new(AliasCommand),
        Box::new(UnaliasCommand),
        Box::new(ExportCommand),
        Box::new(HelpCommand),
        Box::new(EchoCommand),
        Box::new(ExitCommand),
    ];
    for c in cmds {
        builtins.insert(c.name().to_string(), c);
    }
    builtins
}

impl ShellState {
    pub fn execute_builtins(&mut self, cmd: &CommandExecution) -> bool {
        if cmd.args.is_empty() {
            return false;
        }

        let command_name = cmd.args[0].clone();
        if self.builtins.contains_key(&command_name) {
            if let Some(builtin) = self.builtins.remove(&command_name) {
                let res = builtin.execute(self, cmd);
                self.builtins.insert(command_name, builtin);
                return res;
            }
        }
        false
    }
}
