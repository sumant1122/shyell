use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Operator(String),
}

#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub args: Vec<String>,
    pub input_file: Option<String>,
    pub output_file: Option<String>,
    pub append: bool,
    pub bench: bool,
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
        
        if chars[i] == '|' || chars[i] == '<' {
            tokens.push(Token::Operator(chars[i].to_string()));
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
        if chars[i] == '~' && (i + 1 == chars.len() || chars[i + 1].is_whitespace() || chars[i + 1] == '/') {
            if let Some(home) = dirs::home_dir() {
                current_word.push_str(&home.to_string_lossy());
                i += 1;
            }
        }
        
        while i < chars.len() {
            let c = chars[i];
            
            if c == '\\' && !in_single {
                if i + 1 < chars.len() {
                    current_word.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
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
            
            if !in_single && !in_double {
                if c.is_whitespace() || c == '|' || c == '<' || c == '>' {
                    break;
                }
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

pub fn parse_commands(tokens: Vec<Token>) -> Vec<CommandExecution> {
    let mut cmds = Vec::new();
    let mut is_bench = false;
    let mut start_idx = 0;

    if !tokens.is_empty() {
        if let Token::Word(w) = &tokens[0] {
            if w == "bench" {
                is_bench = true;
                start_idx = 1;
            }
        }
    }

    let mut current_cmd = CommandExecution {
        args: Vec::new(),
        input_file: None,
        output_file: None,
        append: false,
        bench: is_bench,
    };
    
    let mut iter = tokens.into_iter().skip(start_idx).peekable();
    while let Some(tok) = iter.next() {
        match tok {
            Token::Operator(op) => {
                match op.as_str() {
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
                    "<" => {
                        if let Some(Token::Word(file)) = iter.next() {
                            current_cmd.input_file = Some(file);
                        }
                    }
                    _ => {}
                }
            }
            Token::Word(w) => {
                current_cmd.args.push(w);
            }
        }
    }
    if !current_cmd.args.is_empty() || current_cmd.input_file.is_some() || current_cmd.output_file.is_some() {
        cmds.push(current_cmd);
    }
    cmds
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_tokenize_quotes() {
        let tokens = tokenize("echo \">\" '<'").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Word("echo".to_string()));
        assert_eq!(tokens[1], Token::Word(">".to_string()));
        assert_eq!(tokens[2], Token::Word("<".to_string()));
    }

    #[test]
    #[serial]
    fn test_tokenize_variables() {
        unsafe { std::env::set_var("TEST_VAR", "123"); }
        let tokens = tokenize("echo $TEST_VAR '$TEST_VAR'").unwrap();
        assert_eq!(tokens[1], Token::Word("123".to_string()));
        assert_eq!(tokens[2], Token::Word("$TEST_VAR".to_string())); // single quotes = no expansion
    }

    #[test]
    fn test_parse_pipeline() {
        let tokens = tokenize("ls | grep rs").unwrap();
        let cmds = parse_commands(tokens);
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].args, vec!["ls"]);
        assert_eq!(cmds[1].args, vec!["grep", "rs"]);
    }
}
