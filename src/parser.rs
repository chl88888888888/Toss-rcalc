#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
    UnaryMinus,
    Modulo,
    Power,
    FunctionCall(String, Vec<Token>), 
}

pub struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while let Some(c) = self.chars.peek() {
            match c {
                ' ' | '\t' | '\n' => {
                    self.chars.next();
                }
                '+' => {
                    tokens.push(Token::Add);
                    self.chars.next();
                }
                '-' => {
                    self.chars.next();

                    let is_unary = tokens.is_empty()
                        || matches!(
                            tokens.last(),
                            Some(Token::Add)
                                | Some(Token::Subtract)
                                | Some(Token::Multiply)
                                | Some(Token::Divide)
                                | Some(Token::LeftParen)
                                | Some(Token::UnaryMinus)
                                | Some(Token::Modulo)
                                | Some(Token::Power)
                        );

                    if is_unary {
                        tokens.push(Token::UnaryMinus);
                    } else {
                        tokens.push(Token::Subtract);
                    }
                }
                '*' => {
                    tokens.push(Token::Multiply);
                    self.chars.next();
                }
                '/' => {
                    tokens.push(Token::Divide);
                    self.chars.next();
                }
                '%' => {
                    tokens.push(Token::Modulo);
                    self.chars.next();
                }
                '^' => {
                    tokens.push(Token::Power);
                    self.chars.next();
                }
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.chars.next();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.chars.next();
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let name = self.parse_identifier();
                    match name.to_lowercase().as_str() {
                        "pi" => tokens.push(Token::Number(std::f64::consts::PI)),
                        "e" => tokens.push(Token::Number(std::f64::consts::E)),
                        _ => {
                            if let Some(&'(') = self.chars.peek() {
                                self.chars.next(); 
                                let args = self.parse_function_args()?;
                                tokens.push(Token::FunctionCall(name, args));
                            } else {
                                return Err(format!("Unexpected identifier: {}", name));
                            }
                        }
                    }
                }
                '0'..='9' | '.' => {
                    let num = self.parse_number()?;
                    tokens.push(Token::Number(num));
                }
                _ => {
                    return Err(format!("Unexpected character: {}", c));
                }
            }
        }
        Ok(tokens)
    }

    fn parse_identifier(&mut self) -> String {
        let mut ident = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                ident.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        ident
    }

    fn parse_function_args(&mut self) -> Result<Vec<Token>, String> {
        let mut args = Vec::new();
        let mut current_arg = String::new();
        let mut paren_count = 0;

        while let Some(&c) = self.chars.peek() {
            match c {
                ')' => {
                    if paren_count == 0 {
                        self.chars.next();
                        if !current_arg.is_empty() {
                            let mut lexer = Lexer::new(&current_arg);
                            let tokens = lexer.tokenize()?;
                            args.extend(tokens);
                        }
                        return Ok(args);
                    } else {
                        paren_count -= 1;
                        current_arg.push(c);
                        self.chars.next();
                    }
                }
                ',' => {
                    if paren_count == 0 {
                        self.chars.next(); 
                        if !current_arg.is_empty() {
                            let mut lexer = Lexer::new(&current_arg);
                            let tokens = lexer.tokenize()?;
                            args.extend(tokens);
                            current_arg.clear();
                        }
                    } else {
                        current_arg.push(c);
                        self.chars.next();
                    }
                }
                '(' => {
                    paren_count += 1;
                    current_arg.push(c);
                    self.chars.next();
                }
                _ => {
                    current_arg.push(c);
                    self.chars.next();
                }
            }
        }

        Err("Unclosed function arguments".to_string())
    }

    fn parse_number(&mut self) -> Result<f64, String> {
        let mut num_str = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() || c == '.' {
                num_str.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        if num_str.is_empty() {
            return Err("Expected number".to_string());
        }
        num_str
            .parse::<f64>()
            .map_err(|_| "Invalid number format".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negative_with_parentheses() {
        let input = "-(-5)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::UnaryMinus,
                Token::LeftParen,
                Token::UnaryMinus,
                Token::Number(5.0),
                Token::RightParen
            ]
        );

        let input = "-(3 + 5)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::UnaryMinus,
                Token::LeftParen,
                Token::Number(3.0),
                Token::Add,
                Token::Number(5.0),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_complex_negatives() {
        let input = "3 + -(-5 * 2)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(3.0),
                Token::Add,
                Token::UnaryMinus, // 负号
                Token::LeftParen,
                Token::UnaryMinus,
                Token::Number(5.0),
                Token::Multiply,
                Token::Number(2.0),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_modulo_and_power() {
        // 取模运算
        let input = "10 % 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![Token::Number(10.0), Token::Modulo, Token::Number(3.0)]
        );

        // 幂运算
        let input = "2 ^ 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![Token::Number(2.0), Token::Power, Token::Number(3.0)]
        );

        // 混合运算
        let input = "2 * 3 ^ 2 % 4";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(2.0),
                Token::Multiply,
                Token::Number(3.0),
                Token::Power,
                Token::Number(2.0),
                Token::Modulo,
                Token::Number(4.0)
            ]
        );
    }
}
