#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
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
                    // Check if it's a negative sign (unary operator)
                    let is_negative = tokens.is_empty() || 
                        matches!(tokens.last(), 
                            Some(Token::Add) | 
                            Some(Token::Subtract) | 
                            Some(Token::Multiply) | 
                            Some(Token::Divide) | 
                            Some(Token::LeftParen));
                    
                    if is_negative {
                        self.chars.next(); // Consume the '-'
                        
                        // Skip any whitespace after the negative sign
                        while let Some(' ' | '\t' | '\n') = self.chars.peek() {
                            self.chars.next();
                        }
                        
                        // Parse the number as negative
                        let num = self.parse_number()?;
                        tokens.push(Token::Number(-num));
                    } else {
                        tokens.push(Token::Subtract);
                        self.chars.next();
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
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.chars.next();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.chars.next();
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
        num_str.parse::<f64>().map_err(|_| "Invalid number format".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let input = "3+5*(2-8)/4";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(3.0),
                Token::Add,
                Token::Number(5.0),
                Token::Multiply,
                Token::LeftParen,
                Token::Number(2.0),
                Token::Subtract,
                Token::Number(8.0),
                Token::RightParen,
                Token::Divide,
                Token::Number(4.0)
            ]
        );
    }
    
    #[test]
    fn test_lexer_with_parentheses() {
        let input = "(3+5)*2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Number(3.0),
                Token::Add,
                Token::Number(5.0),
                Token::RightParen,
                Token::Multiply,
                Token::Number(2.0)
            ]
        );
    }
    
    #[test]
    fn test_negative_numbers() {
        // Negative at start
        let input = "-5 + 3";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(-5.0),
                Token::Add,
                Token::Number(3.0)
            ]
        );
        
        // Negative after operator
        let input = "3 * -5";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(3.0),
                Token::Multiply,
                Token::Number(-5.0)
            ]
        );
        
        // Negative in parentheses
        let input = "(-3 + 5) * 2";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Number(-3.0),
                Token::Add,
                Token::Number(5.0),
                Token::RightParen,
                Token::Multiply,
                Token::Number(2.0)
            ]
        );
        
        // Double negative
        let input = "3 - -5";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(3.0),
                Token::Subtract,
                Token::Number(-5.0)
            ]
        );
    }
    
    #[test]
    fn test_negative_number_errors() {
        // Incomplete expression
        let input = "3 -";
        let mut lexer = Lexer::new(input);
        assert!(lexer.tokenize().is_err());
        
        // Negative sign without number
        let input = "- + 5";
        let mut lexer = Lexer::new(input);
        assert!(lexer.tokenize().is_err());
    }
}