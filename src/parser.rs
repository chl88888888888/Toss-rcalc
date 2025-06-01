#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
    LeftParen,
    RightParen,
    UnaryMinus,  // 新增一元负号运算符
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
                    
                    // 检查是否是一元负号（表达式开头、运算符后或左括号后）
                    let is_unary = tokens.is_empty() || 
                        matches!(tokens.last(), 
                            Some(Token::Add) | 
                            Some(Token::Subtract) | 
                            Some(Token::Multiply) | 
                            Some(Token::Divide) | 
                            Some(Token::LeftParen) |
                            Some(Token::UnaryMinus));
                    
                    if is_unary {
                        // 一元负号：检查后面是数字还是括号
                        if let Some(next_c) = self.chars.peek() {
                            match next_c {
                                '0'..='9' | '.' => {
                                    // 负号后跟数字：解析负数
                                    let num = self.parse_number()?;
                                    tokens.push(Token::Number(-num));
                                }
                                '(' => {
                                    // 负号后跟括号：添加一元负号Token
                                    tokens.push(Token::UnaryMinus);
                                }
                                _ => {
                                    return Err("Expected number or '(' after unary minus".to_string());
                                }
                            }
                        } else {
                            return Err("Unexpected end after '-'".to_string());
                        }
                    } else {
                        // 普通减法运算符
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
    fn test_negative_with_parentheses() {
        let input = "-(-5)";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::UnaryMinus,   // 第一个负号
                Token::LeftParen,
                Token::UnaryMinus,   // 括号内的负号
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
                Token::UnaryMinus,   // 负号
                Token::LeftParen,
                Token::UnaryMinus,   // 括号内的负号
                Token::Number(5.0),
                Token::Multiply,
                Token::Number(2.0),
                Token::RightParen
            ]
        );
    }
}