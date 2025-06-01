use crate::parser::Token;

pub fn evaluate(tokens: &[Token]) -> Result<f64, String> {
    if tokens.is_empty() {
        return Err("Empty expression".to_string());
    }
    
    let mut values: Vec<f64> = Vec::new();
    let mut ops: Vec<Token> = Vec::new();

    for token in tokens {
        match token {
            Token::Number(n) => values.push(*n),
            Token::LeftParen => ops.push(token.clone()),
            Token::RightParen => {
                // Process operations until left parenthesis
                while let Some(op) = ops.last() {
                    if *op == Token::LeftParen {
                        break;
                    }
                    perform_operation(&mut values, &mut ops)?;
                }
                
                // Pop the left parenthesis
                ops.pop().ok_or("Mismatched parentheses".to_string())?;
            }
            Token::Add | Token::Subtract => {
                // Process all higher precedence operators
                while let Some(op) = ops.last() {
                    if matches!(op, Token::Multiply | Token::Divide | Token::Add | Token::Subtract) {
                        perform_operation(&mut values, &mut ops)?;
                    } else {
                        break;
                    }
                }
                ops.push(token.clone());
            }
            Token::Multiply | Token::Divide => {
                // Process same precedence operators
                while let Some(op) = ops.last() {
                    if matches!(op, Token::Multiply | Token::Divide) {
                        perform_operation(&mut values, &mut ops)?;
                    } else {
                        break;
                    }
                }
                ops.push(token.clone());
            }
        }
    }

    // Process remaining operators
    while let Some(_) = ops.last() {
        perform_operation(&mut values, &mut ops)?;
    }

    match values.len() {
        1 => Ok(values[0]),
        0 => Err("No result produced".to_string()),
        _ => Err(format!("Too many values in the stack: {:?}", values)),
    }
}

fn perform_operation(
    values: &mut Vec<f64>,
    ops: &mut Vec<Token>,
) -> Result<(), String> {
    let b = values.pop().ok_or("Missing right operand".to_string())?;
    let a = values.pop().ok_or("Missing left operand".to_string())?;
    let op = ops.pop().ok_or("Missing operator".to_string())?;

    let res = match op {
        Token::Add => a + b,
        Token::Subtract => a - b,
        Token::Multiply => a * b,
        Token::Divide => {
            if b == 0.0 {
                return Err("Division by zero".to_string());
            }
            a / b
        }
        _ => return Err(format!("Unexpected operator: {:?}", op)),
    };

    values.push(res);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Lexer;

    fn eval_expr(expr: &str) -> Result<f64, String> {
        let mut lexer = Lexer::new(expr);
        let tokens = lexer.tokenize()?;
        evaluate(&tokens)
    }

    #[test]
    fn test_basic_operations() {
        assert_eq!(eval_expr("3+5").unwrap(), 8.0);
        assert_eq!(eval_expr("10-3").unwrap(), 7.0);
        assert_eq!(eval_expr("4*6").unwrap(), 24.0);
        assert_eq!(eval_expr("20/5").unwrap(), 4.0);
    }

    #[test]
    fn test_operator_precedence() {
        assert_eq!(eval_expr("3+5*2").unwrap(), 13.0);
        assert_eq!(eval_expr("3*5+2").unwrap(), 17.0);
        assert_eq!(eval_expr("10-8/2").unwrap(), 6.0);
        assert_eq!(eval_expr("3+5*2-8/4").unwrap(), 11.0);
    }

    #[test]
    fn test_division_by_zero() {
        assert!(eval_expr("10/0").is_err());
        assert!(eval_expr("0/0").is_err());
    }
    
    #[test]
    fn test_parentheses() {
        // Basic parentheses
        assert_eq!(eval_expr("(3+5)*2").unwrap(), 16.0);
        assert_eq!(eval_expr("3*(5+2)").unwrap(), 21.0);
        
        // Nested parentheses
        assert_eq!(eval_expr("((3+2)*4)/2").unwrap(), 10.0);
        assert_eq!(eval_expr("(10-(5-2))*3").unwrap(), 21.0);
        
        // Complex expressions
        assert_eq!(eval_expr("(3+5)*(10-4)/3").unwrap(), 16.0);
        assert_eq!(eval_expr("2*(3+4*(5-1))").unwrap(), 38.0);
    }
    
    #[test]
    fn test_parentheses_errors() {
        // Mismatched parentheses
        assert!(eval_expr("(3+5").is_err());
        assert!(eval_expr("3+5)").is_err());
        assert!(eval_expr("((3+5)*2").is_err());
        assert!(eval_expr("(3+5))").is_err());
        
        // Empty parentheses
        assert!(eval_expr("()").is_err());
        assert!(eval_expr("3 + ()").is_err());
    }
    
    #[test]
    fn test_negative_numbers() {
        // Basic negative operations
        assert_eq!(eval_expr("-5 + 3").unwrap(), -2.0);
        assert_eq!(eval_expr("3 * -5").unwrap(), -15.0);
        assert_eq!(eval_expr("10 / -2").unwrap(), -5.0);
        
        // Negative in expressions
        assert_eq!(eval_expr("3 + -5 * 2").unwrap(), -7.0);
        assert_eq!(eval_expr("(3 + -5) * 2").unwrap(), -4.0);
        
        // Double negatives
        assert_eq!(eval_expr("3 - -5").unwrap(), 8.0);
        assert_eq!(eval_expr("-(-5)").unwrap(), 5.0);
        assert_eq!(eval_expr("-(-(-5))").unwrap(), -5.0);
        
        // Complex expressions
        assert_eq!(eval_expr("3 * -(5 + 2)").unwrap(), -21.0);
        assert_eq!(eval_expr("-(3 * 4) + -(10 / 2)").unwrap(), -17.0);
    }
    
    #[test]
    fn test_negative_with_parentheses() {
        assert_eq!(eval_expr("(-3 + 5) * 2").unwrap(), 4.0);
        assert_eq!(eval_expr("-(3 + 5) * 2").unwrap(), -16.0);
        assert_eq!(eval_expr("(3 + 5) * -2").unwrap(), -16.0);
        assert_eq!(eval_expr("-(3 * (5 - 2))").unwrap(), -9.0);
    }
}