use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::env;
use std::fs;
use std::path::Path;

pub struct ShyellHelper {
    pub filename_completer: FilenameCompleter,
    pub builtins: Vec<String>,
}

impl ShyellHelper {
    pub fn new() -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            builtins: vec![
                "cd".into(), "pwd".into(), "sys".into(), "top".into(), 
                "history".into(), "help".into(), "echo".into(), "exit".into(), "bench".into()
            ],
        }
    }

    fn get_path_binaries(&self, prefix: &str) -> Vec<String> {
        let mut binaries = Vec::new();
        if let Ok(path_var) = env::var("PATH") {
            for path in env::split_paths(&path_var) {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with(prefix) {
                            if let Ok(metadata) = entry.metadata() {
                                #[cfg(unix)]
                                {
                                    use std::os::unix::fs::PermissionsExt;
                                    if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                                        binaries.push(name);
                                    }
                                }
                                #[cfg(not(unix))]
                                {
                                    if metadata.is_file() {
                                        binaries.push(name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        binaries.sort();
        binaries.dedup();
        binaries
    }
}

impl Completer for ShyellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let (start, word) = rustyline::completion::extract_word(line, pos, None, |c| c == '|' || c == ' ');
        
        // If it's the first word (or first word after a pipe), complete commands
        let is_command = start == 0 || (start > 1 && line[..start].trim_end().ends_with('|'));

        if is_command {
            let mut matches = Vec::new();
            
            // Complete Built-ins
            for builtin in &self.builtins {
                if builtin.starts_with(word) {
                    matches.push(Pair {
                        display: builtin.clone(),
                        replacement: builtin.clone(),
                    });
                }
            }

            // Complete PATH binaries
            if !word.is_empty() {
                for bin in self.get_path_binaries(word) {
                    matches.push(Pair {
                        display: bin.clone(),
                        replacement: bin.clone(),
                    });
                }
            }

            if !matches.is_empty() {
                return Ok((start, matches));
            }
        }

        // Otherwise, fallback to filename completion
        self.filename_completer.complete(line, pos, ctx)
    }
}

impl Helper for ShyellHelper {}
impl Hinter for ShyellHelper {
    type Hint = String;
}
impl Highlighter for ShyellHelper {}
impl Validator for ShyellHelper {}
