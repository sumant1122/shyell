use crate::monitor::Monitor;
use crate::parser::{ControlOp, PipelineExecution};
use crate::state::ShellState;
use chrono::Duration;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Instant;

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

pub fn get_output_writer(
    output_file: &Option<String>,
    append: bool,
) -> Result<Box<dyn Write>, String> {
    if let Some(file) = output_file {
        let f = if append {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file)
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

pub fn execute_commands(pipelines: Vec<PipelineExecution>, state: &mut ShellState) {
    if pipelines.is_empty() {
        return;
    }

    // Observed Environment: Pre-flight Guardrails
    Monitor::pre_flight_check(state);

    let mut skip_next = false;
    let mut skip_op = ControlOp::None;

    for pipeline in pipelines {
        if skip_next {
            match skip_op {
                ControlOp::And => {
                    if pipeline.control_op != ControlOp::And {
                        skip_next = false;
                    }
                    continue;
                }
                ControlOp::Or => {
                    if pipeline.control_op != ControlOp::Or {
                        skip_next = false;
                    }
                    continue;
                }
                _ => {
                    skip_next = false;
                }
            }
        }

        let cmds = pipeline.commands;
        if cmds.is_empty() {
            continue;
        }

        let is_bench = cmds[0].bench;
        let full_command = cmds
            .iter()
            .map(|c| c.args.join(" "))
            .collect::<Vec<_>>()
            .join(" | ");

        let start_time = if is_bench { Some(Instant::now()) } else { None };

        if cmds.len() == 1 && state.execute_builtins(&cmds[0]) {
            state.last_exit_status = Some(0); // Built-ins handled so far return success
            if let Some(start) = start_time {
                let elapsed = start.elapsed();
                println!(
                    "\x1b[1;35mBench: Built-in command took {:?}\x1b[0m",
                    elapsed
                );
                state.add_benchmark(full_command, elapsed.as_secs_f64(), Some(0));
            }
        } else {
            let mut children = Vec::new();
            let mut previous_stdout: Option<Stdio> = None;
            let cmd_count = cmds.len();
            let mut error_occurred = false;

            for (i, cmd_exec) in cmds.iter().enumerate() {
                if cmd_exec.args.is_empty() {
                    eprintln!("shyell: parse error: empty command in pipeline");
                    state.last_exit_status = Some(1);
                    error_occurred = true;
                    break;
                }

                let stdin = if let Some(ref in_file) = cmd_exec.input_file {
                    match File::open(in_file) {
                        Ok(f) => Stdio::from(f),
                        Err(e) => {
                            eprintln!("shyell: {}: {}", in_file, e);
                            state.last_exit_status = Some(1);
                            error_occurred = true;
                            break;
                        }
                    }
                } else if let Some(prev) = previous_stdout.take() {
                    prev
                } else {
                    Stdio::inherit()
                };

                let stdout = if let Some(ref out_file) = cmd_exec.output_file {
                    let f = if cmd_exec.append {
                        std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(out_file)
                    } else {
                        File::create(out_file)
                    };
                    match f {
                        Ok(f) => Stdio::from(f),
                        Err(e) => {
                            eprintln!("shyell: {}: {}", out_file, e);
                            state.last_exit_status = Some(1);
                            error_occurred = true;
                            break;
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
                        state.last_exit_status = Some(1);
                        error_occurred = true;
                        break;
                    }
                }
            }

            if !error_occurred {
                let mut last_status = None;
                for (name, mut child) in children {
                    match child.wait() {
                        Ok(s) => last_status = Some(s),
                        Err(e) => eprintln!("shyell: error waiting for {}: {}", name, e),
                    }
                }
                state.last_exit_status = last_status.and_then(|s| s.code()).or(Some(0));
            }

            if let Some(start) = start_time {
                let elapsed = start.elapsed();
                let exit_code = state.last_exit_status;

                println!("\x1b[1;35m--- Benchmark Results ---\x1b[0m");
                println!("{:<15} {:?}", "Execution Time:", elapsed);
                if let Some(code) = exit_code {
                    println!("{:<15} {}", "Exit Status:", code);
                }

                Monitor::check_regression(state, &full_command, elapsed.as_secs_f64());
                state.add_benchmark(full_command, elapsed.as_secs_f64(), exit_code);
            }
        }

        match pipeline.control_op {
            ControlOp::And => {
                if state.last_exit_status != Some(0) {
                    skip_next = true;
                    skip_op = ControlOp::And;
                }
            }
            ControlOp::Or => {
                if state.last_exit_status == Some(0) {
                    skip_next = true;
                    skip_op = ControlOp::Or;
                }
            }
            ControlOp::Semi | ControlOp::None => {
                skip_next = false;
            }
        }
    }
}
