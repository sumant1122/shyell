use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;
use chrono::Duration;
use crate::state::ShellState;
use crate::parser::CommandExecution;
use crate::monitor::Monitor;

pub fn format_duration(seconds: u64) -> String {
    let d = Duration::seconds(seconds as i64);
    let hours = d.num_hours();
    let mins = d.num_minutes() % 60;
    let secs = d.num_seconds() % 60;
    if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn get_output_writer(output_file: &Option<String>, append: bool) -> Result<Box<dyn Write>, String> {
    if let Some(file) = output_file {
        let f = if append {
            std::fs::OpenOptions::new().create(true).append(true).open(file)
        } else {
            File::create(file)
        };
        match f {
            Ok(f) => Ok(Box::new(f)),
            Err(e) => Err(format!("{}: {}", file, e)),
        }
    } else {
        Ok(Box::new(std::io::stdout()))
    }
}

pub fn execute_commands(cmds: Vec<CommandExecution>, state: &mut ShellState) {
    if cmds.is_empty() {
        return;
    }

    // Observed Environment: Pre-flight Guardrails
    Monitor::pre_flight_check(state);

    let is_bench = cmds[0].bench;
    let full_command = cmds.iter()
        .map(|c| c.args.join(" "))
        .collect::<Vec<_>>()
        .join(" | ");

    let start_time = if is_bench { Some(Instant::now()) } else { None };

    if cmds.len() == 1 && state.execute_builtins(&cmds[0]) {
        if let Some(start) = start_time {
            let elapsed = start.elapsed();
            println!("\x1b[1;35mBench: Built-in command took {:?}\x1b[0m", elapsed);
            state.add_benchmark(full_command, elapsed.as_secs_f64(), Some(0));
        }
        return;
    }

    let mut children = Vec::new();
    let mut previous_stdout: Option<Stdio> = None;
    let cmd_count = cmds.len();

    for (i, cmd_exec) in cmds.iter().enumerate() {
        if cmd_exec.args.is_empty() {
            eprintln!("shyell: parse error: empty command in pipeline");
            return;
        }

        let stdin = if let Some(ref in_file) = cmd_exec.input_file {
            match File::open(in_file) {
                Ok(f) => Stdio::from(f),
                Err(e) => {
                    eprintln!("shyell: {}: {}", in_file, e);
                    return;
                }
            }
        } else if let Some(prev) = previous_stdout.take() {
            prev
        } else {
            Stdio::inherit()
        };

        let stdout = if let Some(ref out_file) = cmd_exec.output_file {
            let f = if cmd_exec.append {
                std::fs::OpenOptions::new().create(true).append(true).open(out_file)
            } else {
                File::create(out_file)
            };
            match f {
                Ok(f) => Stdio::from(f),
                Err(e) => {
                    eprintln!("shyell: {}: {}", out_file, e);
                    return;
                }
            }
        } else if i < cmd_count - 1 {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };

        let command = &cmd_exec.args[0];
        let args = &cmd_exec.args[1..];

        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.stdin(stdin);
        cmd.stdout(stdout);

        match cmd.spawn() {
            Ok(mut child) => {
                if i < cmd_count - 1 {
                    previous_stdout = Some(Stdio::from(child.stdout.take().unwrap()));
                }
                children.push((command.clone(), child));
            }
            Err(e) => {
                eprintln!("shyell: {}: {}", command, e);
                break;
            }
        }
    }

    let mut last_status = None;
    for (name, mut child) in children {
        match child.wait() {
            Ok(s) => last_status = Some(s),
            Err(e) => eprintln!("shyell: error waiting for {}: {}", name, e),
        }
    }

    if let Some(start) = start_time {
        let elapsed = start.elapsed();
        let exit_code = last_status.and_then(|s| s.code());
        
        println!("\x1b[1;35m--- Benchmark Results ---\x1b[0m");
        println!("{:<15} {:?}", "Execution Time:", elapsed);
        if let Some(code) = exit_code {
            println!("{:<15} {}", "Exit Status:", code);
        }

        // Observed Environment: Performance Regression Alerts
        Monitor::check_regression(state, &full_command, elapsed.as_secs_f64());
        
        // Observed Environment: Flight Recorder
        state.add_benchmark(full_command, elapsed.as_secs_f64(), exit_code);
    }
}
