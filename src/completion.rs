use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::env;
use std::fs;
use std::cell::RefCell;

pub struct VantageHelper {
    pub filename_completer: FilenameCompleter,
    pub builtins: Vec<String>,
    pub path_cache: RefCell<Option<(String, Vec<String>)>>,
}

impl VantageHelper {
    pub fn new() -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            builtins: vec![
                "cd".into(), "pwd".into(), "sys".into(), "top".into(), 
                "history".into(), "help".into(), "echo".into(), "exit".into(), "bench".into()
            ],
            path_cache: RefCell::new(None),
        }
    }

    fn get_path_binaries(&self, prefix: &str) -> Vec<String> {
        let current_path = env::var("PATH").unwrap_or_default();
        let mut update_needed = true;

        if let Some((cached_path, _)) = &*self.path_cache.borrow() {
            if current_path == *cached_path {
                update_needed = false;
            }
        }

        if update_needed {
            let mut binaries = Vec::new();
            for path in env::split_paths(&current_path) {
                if let Ok(entries) = fs::read_dir(path) {
                    for entry in entries.flatten() {
                        if let Ok(metadata) = entry.metadata() {
                            let name = entry.file_name().to_string_lossy().to_string();
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
            binaries.sort();
            binaries.dedup();
            *self.path_cache.borrow_mut() = Some((current_path, binaries));
        }

        let cache = self.path_cache.borrow();
        let binaries = &cache.as_ref().unwrap().1;
        binaries.iter().filter(|b| b.starts_with(prefix)).cloned().collect()
    }
}

impl Completer for VantageHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let (start, word) = rustyline::completion::extract_word(line, pos, None, |c| c == '|' || c == ' ');
        
        let is_command = start == 0 || (start > 1 && line[..start].trim_end().ends_with('|'));

        if is_command {
            let mut matches = Vec::new();
            
            for builtin in &self.builtins {
                if builtin.starts_with(word) {
                    matches.push(Pair {
                        display: builtin.clone(),
                        replacement: builtin.clone(),
                    });
                }
            }

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

        self.filename_completer.complete(line, pos, ctx)
    }
}

impl Helper for VantageHelper {}
impl Hinter for VantageHelper {
    type Hint = String;
}
impl Highlighter for VantageHelper {}
impl Validator for VantageHelper {}
