use std::env;
use std::io::Write;
use std::path::PathBuf;
use sysinfo::{ProcessRefreshKind, RefreshKind, System, Disks};
use crate::state::ShellState;
use crate::parser::CommandExecution;
use crate::executor::{get_output_writer, format_duration};

impl ShellState {
    pub fn execute_builtins(&mut self, cmd: &CommandExecution) -> bool {
        if cmd.args.is_empty() {
            return false;
        }
        
        let command = cmd.args[0].as_str();
        
        match command {
            "cd" => {
                let target = if cmd.args.len() < 2 {
                    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
                } else if cmd.args[1] == "-" {
                    if let Some(prev) = &self.prev_dir {
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
                        self.prev_dir = Some(current.clone());
                        unsafe {
                            env::set_var("OLDPWD", current);
                            if let Ok(new_dir) = env::current_dir() {
                                env::set_var("PWD", new_dir);
                            }
                        }
                    }
                    Err(e) => eprintln!("cd: {}: {}", target.display(), e),
                }
                true
            }
            "pwd" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("vantage: {}", e); return true; }
                };
                match env::current_dir() {
                    Ok(dir) => writeln!(writer, "{}", dir.display()).unwrap_or(()),
                    Err(e) => eprintln!("pwd: {}", e),
                }
                true
            }
            "sys" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("vantage: {}", e); return true; }
                };
                
                writeln!(writer, "\x1b[1;36m--- System Status ---\x1b[0m").unwrap_or(());
                writeln!(writer, "{:<15} {}", "OS:", System::name().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
                writeln!(writer, "{:<15} {}", "Kernel:", System::kernel_version().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
                writeln!(writer, "{:<15} {}", "Hostname:", System::host_name().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
                writeln!(writer, "{:<15} {}", "Uptime:", format_duration(System::uptime())).unwrap_or(());
                
                let total_mem = self.sys.total_memory() / 1024 / 1024;
                let used_mem = self.sys.used_memory() / 1024 / 1024;
                let mem_pct = (used_mem as f64 / total_mem as f64 * 100.0) as usize;
                
                let bar_len = 20;
                let filled = (mem_pct * bar_len) / 100;
                let bar = format!("[{}{}]", "#".repeat(filled), ".".repeat(bar_len - filled));
                
                writeln!(writer, "{:<15} {} {}% ({}MB / {}MB)", "Memory:", bar, mem_pct, used_mem, total_mem).unwrap_or(());
                writeln!(writer, "{:<15} {:.2}%", "CPU Load:", self.sys.global_cpu_usage()).unwrap_or(());

                let disks = Disks::new_with_refreshed_list();
                if let Some(root) = disks.iter().find(|d| d.mount_point() == std::path::Path::new("/")) {
                    let total = root.total_space() / 1024 / 1024 / 1024;
                    let avail = root.available_space() / 1024 / 1024 / 1024;
                    let used = total - avail;
                    let disk_pct = (used as f64 / total as f64 * 100.0) as usize;
                    let filled_disk = (disk_pct * bar_len) / 100;
                    let bar_disk = format!("[{}{}]", "#".repeat(filled_disk), ".".repeat(bar_len - filled_disk));
                    writeln!(writer, "{:<15} {} {}% ({}GB / {}GB)", "Disk (/):", bar_disk, disk_pct, used, total).unwrap_or(());
                }
                
                true
            }
            "top" => {
                self.sys.refresh_specifics(
                    RefreshKind::nothing().with_processes(ProcessRefreshKind::everything())
                );
                
                let mut processes: Vec<_> = self.sys.processes().values().collect();
                processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
                
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("vantage: {}", e); return true; }
                };
                
                writeln!(writer, "\x1b[1;33m{:<8} {:<15} {:<10} {:<10}\x1b[0m", "PID", "Name", "CPU %", "Mem MB").unwrap_or(());
                for p in processes.iter().take(10) {
                    writeln!(writer, "{:<8} {:<15} {:<10.2} {:<10}", 
                        p.pid(), 
                        p.name().to_string_lossy(), 
                        p.cpu_usage(), 
                        p.memory() / 1024 / 1024
                    ).unwrap_or(());
                }
                true
            }
            "history" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("vantage: {}", e); return true; }
                };
                writeln!(writer, "\x1b[1;35m--- Performance History ---\x1b[0m").unwrap_or(());
                for r in self.bench_results.iter().rev().take(20) {
                    writeln!(writer, "[{}] {:<20} | {:.2}s | Exit: {:?}", 
                        r.timestamp.format("%Y-%m-%d %H:%M:%S"), 
                        r.command, 
                        r.duration_secs, 
                        r.exit_status
                    ).unwrap_or(());
                }
                true
            }
            "alias" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("vantage: {}", e); return true; }
                };
                if cmd.args.len() < 2 {
                    for (k, v) in &self.aliases {
                        writeln!(writer, "alias {}='{}'", k, v).unwrap_or(());
                    }
                } else {
                    let arg = &cmd.args[1];
                    if let Some(idx) = arg.find('=') {
                        let key = &arg[..idx];
                        let val = &arg[idx + 1..];
                        self.aliases.insert(key.to_string(), val.to_string());
                        self.save_aliases();
                    } else {
                        if let Some(v) = self.aliases.get(arg) {
                            writeln!(writer, "alias {}='{}'", arg, v).unwrap_or(());
                        } else {
                            eprintln!("alias: {}: not found", arg);
                        }
                    }
                }
                true
            }
            "unalias" => {
                if cmd.args.len() < 2 {
                    eprintln!("unalias: usage: unalias name");
                } else {
                    let key = &cmd.args[1];
                    if self.aliases.remove(key).is_some() {
                        self.save_aliases();
                    } else {
                        eprintln!("unalias: {}: not found", key);
                    }
                }
                true
            }
            "export" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("vantage: {}", e); return true; }
                };
                if cmd.args.len() < 2 {
                    for (k, v) in env::vars() {
                        writeln!(writer, "{}={}", k, v).unwrap_or(());
                    }
                } else {
                    let arg = &cmd.args[1];
                    if let Some(idx) = arg.find('=') {
                        let key = &arg[..idx];
                        let val = &arg[idx + 1..];
                        unsafe { env::set_var(key, val); }
                    } else {
                        eprintln!("export: usage: export KEY=VALUE");
                    }
                }
                true
            }
            "help" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("vantage: {}", e); return true; }
                };
                writeln!(writer, "\x1b[1;32mVantage - Advanced Performance Shell\x1b[0m").unwrap_or(());
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
            "echo" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("vantage: {}", e); return true; }
                };
                let output = cmd.args[1..].join(" ");
                writeln!(writer, "{}", output).unwrap_or(());
                true
            }
            "exit" => {
                std::process::exit(0);
            }
            _ => false,
        }
    }
}
