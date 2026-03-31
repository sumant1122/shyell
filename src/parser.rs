use std::env;
use crate::state::expand_word;

#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub args: Vec<String>,
    pub input_file: Option<String>,
    pub output_file: Option<String>,
    pub append: bool,
    pub bench: bool,
}

pub fn parse_commands(words: Vec<String>) -> Vec<CommandExecution> {
    let mut cmds = Vec::new();
    let mut is_bench = false;
    let mut start_idx = 0;

    if !words.is_empty() && words[0] == "bench" {
        is_bench = true;
        start_idx = 1;
    }

    let mut current_cmd = CommandExecution {
        args: Vec::new(),
        input_file: None,
        output_file: None,
        append: false,
        bench: is_bench,
    };
    
    let mut iter = words.into_iter().skip(start_idx).peekable();
    while let Some(word) = iter.next() {
        match word.as_str() {
            "|" => {
                cmds.push(current_cmd);
                current_cmd = CommandExecution {
                    args: Vec::new(),
                    input_file: None,
                    output_file: None,
                    append: false,
                    bench: is_bench,
                };
            }
            ">" => {
                if let Some(file) = iter.next() {
                    current_cmd.output_file = Some(expand_word(&file));
                }
            }
            ">>" => {
                if let Some(file) = iter.next() {
                    current_cmd.output_file = Some(expand_word(&file));
                    current_cmd.append = true;
                }
            }
            "<" => {
                if let Some(file) = iter.next() {
                    current_cmd.input_file = Some(expand_word(&file));
                }
            }
            _ => {
                current_cmd.args.push(expand_word(&word));
            }
        }
    }
    if !current_cmd.args.is_empty() || current_cmd.input_file.is_some() || current_cmd.output_file.is_some() {
        cmds.push(current_cmd);
    }
    cmds
}
