use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Operator(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlOp {
    And,
    Or,
    Semi,
    Background,
    None,
}

#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub args: Vec<String>,
    pub input_file: Option<String>,
    pub output_file: Option<String>,
    pub stderr_file: Option<String>,
    pub append: bool,
    pub stderr_append: bool,
    pub bench: bool,
}

#[derive(Debug, Clone)]
pub struct PipelineExecution {
    pub commands: Vec<CommandExecution>,
    pub control_op: ControlOp,
}

pub fn tokenize(line: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        // Check for 2> or 2>>
        if chars[i] == '2' && i + 1 < chars.len() && chars[i + 1] == '>' {
             if i + 2 < chars.len() && chars[i + 2] == '>' {
                tokens.push(Token::Operator("2>>".to_string()));
                i += 3;
            } else {
                tokens.push(Token::Operator("2>".to_string()));
                i += 2;
            }
            continue;
        }

        // Check for &> (redirection for both)
        if chars[i] == '&' && i + 1 < chars.len() && chars[i + 1] == '>' {
            tokens.push(Token::Operator("&>".to_string()));
            i += 2;
            continue;
        }

        if chars[i] == '&' {
            if i + 1 < chars.len() && chars[i + 1] == '&' {
                tokens.push(Token::Operator("&&".to_string()));
                i += 2;
            } else {
                tokens.push(Token::Operator("&".to_string()));
                i += 1;
            }
            continue;
        }

        if chars[i] == '|' {
            if i + 1 < chars.len() && chars[i + 1] == '|' {
                tokens.push(Token::Operator("||".to_string()));
                i += 2;
                continue;
            } else {
                tokens.push(Token::Operator("|".to_string()));
                i += 1;
                continue;
            }
        }
        if chars[i] == ';' {
            tokens.push(Token::Operator(";".to_string()));
            i += 1;
            continue;
        }
        if chars[i] == '<' {
            tokens.push(Token::Operator("<".to_string()));
            i += 1;
            continue;
        }

        if chars[i] == '>' {
            if i + 1 < chars.len() && chars[i + 1] == '>' {
                tokens.push(Token::Operator(">>".to_string()));
                i += 2;
            } else {
                tokens.push(Token::Operator(">".to_string()));
                i += 1;
            }
            continue;
        }

        let mut current_word = String::new();
        let mut in_single = false;
        let mut in_double = false;

        // Unquoted tilde expansion at start of word
        if chars[i] == '~'
            && (i + 1 == chars.len() || chars[i + 1].is_whitespace() || chars[i + 1] == '/')
            && let Some(home) = dirs::home_dir()
        {
            current_word.push_str(&home.to_string_lossy());
            i += 1;
        }

        while i < chars.len() {
            let c = chars[i];

            if c == '\\' && !in_single && i + 1 < chars.len() {
                current_word.push(chars[i + 1]);
                i += 2;
                continue;
            }

            if c == '\'' && !in_double {
                in_single = !in_single;
                i += 1;
                continue;
            }

            if c == '"' && !in_single {
                in_double = !in_double;
                i += 1;
                continue;
            }

            if !in_single
                && !in_double
                && (c.is_whitespace() || c == '|' || c == '<' || c == '>' || c == '&' || c == ';')
            {
                break;
            }

            if c == '$' && !in_single {
                let mut var = String::new();
                i += 1;
                if i < chars.len() && chars[i] == '{' {
                    i += 1;
                    while i < chars.len() && chars[i] != '}' {
                        var.push(chars[i]);
                        i += 1;
                    }
                    if i < chars.len() && chars[i] == '}' {
                        i += 1; // consume '}'
                    }
                } else {
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        var.push(chars[i]);
                        i += 1;
                    }
                }
                if !var.is_empty() {
                    if let Ok(val) = env::var(&var) {
                        current_word.push_str(&val);
                    }
                } else {
                    current_word.push('$');
                }
                continue;
            }

            current_word.push(c);
            i += 1;
        }

        if in_single || in_double {
            return Err("Unclosed quote".to_string());
        }

        tokens.push(Token::Word(current_word));
    }
    Ok(tokens)
}

pub fn parse_commands(
    tokens: Vec<Token>,
    aliases: &HashMap<String, String>,
) -> Vec<PipelineExecution> {
    let mut expanded_tokens = Vec::new();
    let mut is_first = true;
    for token in tokens {
        if let Token::Word(ref w) = token
            && is_first
        {
            if w == "bench" {
                expanded_tokens.push(token.clone());
                continue;
            } else if let Some(alias_val) = aliases.get(w)
                && let Ok(alias_tokens) = tokenize(alias_val)
            {
                expanded_tokens.extend(alias_tokens);
                is_first = false;
                continue;
            }
        }

        match &token {
            Token::Operator(op) if op == "|" || op == "&&" || op == "||" || op == ";" || op == "&" => {
                is_first = true;
            }
            Token::Word(_) => {
                is_first = false;
            }
            _ => {}
        }
        expanded_tokens.push(token);
    }

    let mut pipelines = Vec::new();
    let mut current_pipeline = Vec::new();

    let mut current_cmd = CommandExecution {
        args: Vec::new(),
        input_file: None,
        output_file: None,
        stderr_file: None,
        append: false,
        stderr_append: false,
        bench: false,
    };

    let mut iter = expanded_tokens.into_iter().peekable();
    let mut expecting_new_command = true;

    while let Some(tok) = iter.next() {
        if expecting_new_command {
            if let Token::Word(w) = &tok
                && w == "bench"
            {
                current_cmd.bench = true;
                continue;
            }
            expecting_new_command = false;
        }

        match tok {
            Token::Operator(op) => match op.as_str() {
                "|" => {
                    current_pipeline.push(current_cmd);
                    current_cmd = CommandExecution {
                        args: Vec::new(),
                        input_file: None,
                        output_file: None,
                        stderr_file: None,
                        append: false,
                        stderr_append: false,
                        bench: false,
                    };
                    expecting_new_command = true;
                }
                "&&" | "||" | ";" | "&" => {
                    if !current_cmd.args.is_empty()
                        || current_cmd.input_file.is_some()
                        || current_cmd.output_file.is_some()
                        || current_cmd.stderr_file.is_some()
                    {
                        current_pipeline.push(current_cmd);
                    }

                    let control_op = match op.as_str() {
                        "&&" => ControlOp::And,
                        "||" => ControlOp::Or,
                        ";" => ControlOp::Semi,
                        "&" => ControlOp::Background,
                        _ => ControlOp::None,
                    };

                    if !current_pipeline.is_empty() {
                        pipelines.push(PipelineExecution {
                            commands: current_pipeline,
                            control_op,
                        });
                    }

                    current_pipeline = Vec::new();
                    current_cmd = CommandExecution {
                        args: Vec::new(),
                        input_file: None,
                        output_file: None,
                        stderr_file: None,
                        append: false,
                        stderr_append: false,
                        bench: false,
                    };
                    expecting_new_command = true;
                }
                ">" => {
                    if let Some(Token::Word(file)) = iter.next() {
                        current_cmd.output_file = Some(file);
                    }
                }
                ">>" => {
                    if let Some(Token::Word(file)) = iter.next() {
                        current_cmd.output_file = Some(file);
                        current_cmd.append = true;
                    }
                }
                "2>" => {
                    if let Some(Token::Word(file)) = iter.next() {
                        current_cmd.stderr_file = Some(file);
                    }
                }
                "2>>" => {
                    if let Some(Token::Word(file)) = iter.next() {
                        current_cmd.stderr_file = Some(file);
                        current_cmd.stderr_append = true;
                    }
                }
                "&>" => {
                    if let Some(Token::Word(file)) = iter.next() {
                        current_cmd.output_file = Some(file.clone());
                        current_cmd.stderr_file = Some(file);
                    }
                }
                "<" => {
                    if let Some(Token::Word(file)) = iter.next() {
                        current_cmd.input_file = Some(file);
                    }
                }
                _ => {}
            },
            Token::Word(w) => {
                current_cmd.args.push(w);
            }
        }
    }

    if !current_cmd.args.is_empty()
        || current_cmd.input_file.is_some()
        || current_cmd.output_file.is_some()
        || current_cmd.stderr_file.is_some()
    {
        current_pipeline.push(current_cmd);
    }

    if !current_pipeline.is_empty() {
        pipelines.push(PipelineExecution {
            commands: current_pipeline,
            control_op: ControlOp::None,
        });
    }

    pipelines
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_quotes() {
        let tokens = tokenize("echo \">\" '<'").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word(">".to_string()));
        assert_eq!(tokens[2], Token::Word("<".to_string()));
    }

    #[test]
    fn test_parse_pipeline() {
        let tokens = tokenize("ls | grep rs").unwrap();
        let aliases = HashMap::new();
        let pipelines = parse_commands(tokens, &aliases);
        assert_eq!(pipelines.len(), 1);
        let cmds = &pipelines[0].commands;
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].args, vec!["ls"]);
        assert_eq!(cmds[1].args, vec!["grep", "rs"]);
    }

    #[test]
    fn test_parse_redirections() {
        let tokens = tokenize("ls 2> err.log &> both.log").unwrap();
        let aliases = HashMap::new();
        let pipelines = parse_commands(tokens, &aliases);
        assert_eq!(pipelines.len(), 1);
        let cmd = &pipelines[0].commands[0];
        assert_eq!(cmd.stderr_file, Some("both.log".to_string()));
        assert_eq!(cmd.output_file, Some("both.log".to_string()));
    }

    #[test]
    fn test_parse_background() {
        let tokens = tokenize("sleep 10 &").unwrap();
        let aliases = HashMap::new();
        let pipelines = parse_commands(tokens, &aliases);
        assert_eq!(pipelines.len(), 1);
        assert_eq!(pipelines[0].control_op, ControlOp::Background);
    }
}
