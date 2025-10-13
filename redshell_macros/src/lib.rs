extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

#[derive(Default)]
struct ParserCmd {
    args: Vec<String>,
    redirections: Vec<(i32, String)>,
    word: String,
    is_quoted: bool,
    redirect_fd: Option<i32>,
    format_word: bool,
}

impl ParserCmd {
    fn eat_word(&mut self) {
        if !self.word.is_empty() {
            let expression = if self.format_word {
                format!("format!(\"{}\")", self.word)
            }
            else {
                format!("\"{}\"", self.word)
            };

            if let Some(redirect_fd) = self.redirect_fd {
                self.redirections.push((redirect_fd, expression));
            }
            else {
                self.args.push(expression);
            }
            self.word.clear();
        }
        self.format_word = false;
    }

    fn parse_token(&mut self, token: char) {
        match token {
            ' '|'\t'|'\n' => {
                if self.is_quoted {
                    self.word.push(token);
                }
                else {
                    self.eat_word();
                }
            },
            '\'' => {
                if self.is_quoted {
                    self.eat_word();
                }
                self.is_quoted = !self.is_quoted;
            },
            '{' => {
                self.word.push(token);
                self.format_word = true;
            },
            '>'|'<' => {
                if self.is_quoted {
                    self.word.push(token);
                }
                else {
                    if self.word.is_empty() {
                        self.redirect_fd = Some(1);
                    }
                    else {
                        let fd : i32 = self.word.parse()
                            .expect("'>' must be preceded by the file descriptor number");
                        self.redirect_fd = Some(fd);
                    }
                    self.word.clear();
                }
            },
            c => {
                self.word.push(c);
            }
        }
    }

    fn parse_expression(mut self, expression: &str) -> String {
        let tokens = expression.chars();
        for token in tokens {
            self.parse_token(token);
        }
        self.eat_word();

        let mut args = self.args.iter();
        let arg0 = args.next().expect("No command argument");
        println!("cmd.expression={expression}");
        println!("cmd.arg0={arg0}");
        println!("cmd.args={args:?}");
        println!("cmd.redirections={:?}", self.redirections);

        let mut source_code = String::new();
        source_code += "{";
        source_code += &format!("let mut cmd = std::process::Command::new({arg0});");
        for arg in args {
            source_code += &format!("cmd.arg({arg});");
        }
        for (fd, filename) in &self.redirections {
            source_code += &format!("cmd.redirect({fd}, {filename});");
        }
        source_code += "cmd}";
        println!("cmd.source={source_code}");
        source_code
    }
}

fn parse_cmd_expression(expression: &str) -> String {
    ParserCmd::default()
        .parse_expression(expression)
}

#[proc_macro]
pub fn cmd(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let expression = input.value();
    let source_code = parse_cmd_expression(&expression);
    source_code.parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cmd_basic() {
        let source_code = parse_cmd_expression("make -C src/ all");
        let expected = "{let mut cmd = std::process::Command::new(\"make\");cmd.arg(\"-C\");cmd.arg(\"src/\");cmd.arg(\"all\");cmd}";
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression(" make   -C   src/ all");
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression("\tmake   -C src/   all");
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression("make   -C \tsrc/ all ");
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression("make -C 'src/' all");
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression("make   -C  'src/' ''  all");
        assert_eq!(source_code, expected);
    }

    #[test]
    fn parse_cmd_with_quotes() {
        let source_code = parse_cmd_expression("make -C 'Work Dir/src/' all");
        let expected = "{let mut cmd = std::process::Command::new(\"make\");cmd.arg(\"-C\");cmd.arg(\"Work Dir/src/\");cmd.arg(\"all\");cmd}";
        assert_eq!(source_code, expected);
    }

    #[test]
    fn parse_cmd_with_var() {
        let source_code = parse_cmd_expression("make -C {dir} all");
        let expected = "{let mut cmd = std::process::Command::new(\"make\");cmd.arg(\"-C\");cmd.arg(format!(\"{dir}\"));cmd.arg(\"all\");cmd}";
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression("make -C {dir}/src all");
        let expected = "{let mut cmd = std::process::Command::new(\"make\");cmd.arg(\"-C\");cmd.arg(format!(\"{dir}/src\"));cmd.arg(\"all\");cmd}";
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression("make -C Workspace/{dir}/src all");
        let expected = "{let mut cmd = std::process::Command::new(\"make\");cmd.arg(\"-C\");cmd.arg(format!(\"Workspace/{dir}/src\"));cmd.arg(\"all\");cmd}";
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression("make -C 'Work space/{dir}' all");
        let expected = "{let mut cmd = std::process::Command::new(\"make\");cmd.arg(\"-C\");cmd.arg(format!(\"Work space/{dir}\"));cmd.arg(\"all\");cmd}";
        assert_eq!(source_code, expected);
    }

    #[test]
    fn parse_cmd_with_multiple_var() {
        let source_code = parse_cmd_expression("make -C {dir}/{subdir} all");
        let expected = "{let mut cmd = std::process::Command::new(\"make\");cmd.arg(\"-C\");cmd.arg(format!(\"{dir}/{subdir}\"));cmd.arg(\"all\");cmd}";
        assert_eq!(source_code, expected);

        let source_code = parse_cmd_expression("make -C {dir}/{subdir} {target}");
        let expected = "{let mut cmd = std::process::Command::new(\"make\");cmd.arg(\"-C\");cmd.arg(format!(\"{dir}/{subdir}\"));cmd.arg(format!(\"{target}\"));cmd}";
        assert_eq!(source_code, expected);
    }

}
