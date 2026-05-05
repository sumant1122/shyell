use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use std::borrow::Cow;
use std::cell::RefCell;
use std::env;
use std::fs;

pub struct ShyellHelper {
    pub filename_completer: FilenameCompleter,
    pub builtins: Vec<String>,
    pub path_cache: RefCell<Option<(String, Vec<String>)>>,
}

impl ShyellHelper {
    pub fn new() -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            builtins: vec![
                "cd".into(),
                "pwd".into(),
                "sys".into(),
                "top".into(),
                "history".into(),
                "help".into(),
                "echo".into(),
                "exit".into(),
                "bench".into(),
                "alias".into(),
                "unalias".into(),
                "export".into(),
            ],
            path_cache: RefCell::new(None),
        }
    }

    fn get_path_binaries(&self, prefix: &str) -> Vec<String> {
        let current_path = env::var("PATH").unwrap_or_default();
        let mut update_needed = true;

        if let Some((cached_path, _)) = &*self.path_cache.borrow()
            && current_path == *cached_path
        {
            update_needed = false;
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
                                if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
                                {
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
        binaries
            .iter()
            .filter(|b| b.starts_with(prefix))
            .cloned()
            .collect()
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
        let (start, word) =
            rustyline::completion::extract_word(line, pos, None, |c| c == '|' || c == ' ');

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

        // Context-aware completion: if command is 'cd', only show directories
        if line[..start].trim() == "cd" {
             let mut matches = Vec::new();
             if let Ok(entries) = fs::read_dir(env::current_dir().unwrap_or_else(|_| ".".into())) {
                 for entry in entries.flatten() {
                     if let Ok(metadata) = entry.metadata() {
                         if metadata.is_dir() {
                             let name = entry.file_name().to_string_lossy().to_string();
                             if name.starts_with(word) {
                                 matches.push(Pair {
                                     display: name.clone(),
                                     replacement: name,
                                 });
                             }
                         }
                     }
                 }
             }
             if !matches.is_empty() {
                 return Ok((start, matches));
             }
        }

        self.filename_completer.complete(line, pos, ctx)
    }
}


impl Helper for ShyellHelper {}

impl Hinter for ShyellHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if line.is_empty() || pos < line.len() {
            return None;
        }

        let word = line.split_whitespace().last().unwrap_or("");
        if line.len() == word.len() {
            // Only hint if it's the first word
            for builtin in &self.builtins {
                if builtin.starts_with(word) && builtin != word {
                    return Some(builtin[word.len()..].to_string());
                }
            }
        }
        None
    }
}

impl Highlighter for ShyellHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        for builtin in &self.builtins {
            if line.starts_with(builtin)
                && (line.len() == builtin.len()
                    || line.as_bytes()[builtin.len()].is_ascii_whitespace())
            {
                let highlighted = format!("\x1b[32m{}\x1b[0m{}", builtin, &line[builtin.len()..]);
                return Cow::Owned(highlighted);
            }
        }
        Cow::Borrowed(line)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{}\x1b[0m", hint))
    }
}

impl Validator for ShyellHelper {}
