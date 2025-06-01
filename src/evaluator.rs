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
                while let Some(op) = ops.last() {
                    if *op == Token::LeftParen {
                        break;
                    }
                    perform_operation(&mut values, &mut ops)?;
                }
                ops.pop().ok_or("Mismatched parentheses".to_string())?;
            }
            Token::UnaryMinus => {
                // 处理一元负号：直接应用到下一个操作数
                ops.push(token.clone());
            }
            Token::Add | Token::Subtract => {
                // 处理一元负号（如果有）
                while let Some(op) = ops.last() {
                    if let Token::UnaryMinus = op {
                        apply_unary_minus(&mut values)?;
                        ops.pop();
                    } else {
                        break;
                    }
                }
                
                // 处理其他运算符
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
                // 处理一元负号（如果有）
                while let Some(op) = ops.last() {
                    if let Token::UnaryMinus = op {
                        apply_unary_minus(&mut values)?;
                        ops.pop();
                    } else {
                        break;
                    }
                }
                
                // 处理乘除运算符
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
    
    // 处理剩余的一元负号
    while let Some(op) = ops.last() {
        if let Token::UnaryMinus = op {
            apply_unary_minus(&mut values)?;
            ops.pop();
        } else {
            break;
        }
    }

    // 处理剩余的二元运算符
    while let Some(_) = ops.last() {
        perform_operation(&mut values, &mut ops)?;
    }

    match values.len() {
        1 => Ok(values[0]),
        0 => Err("No result produced".to_string()),
        _ => Err(format!("Too many values in the stack: {:?}", values)),
    }
}

// 应用一元负号到栈顶操作数
fn apply_unary_minus(values: &mut Vec<f64>) -> Result<(), String> {
    let value = values.pop().ok_or("Missing operand for unary minus".to_string())?;
    values.push(-value);
    Ok(())
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
    fn test_unary_minus() {
        // 基本一元负号
        assert_eq!(eval_expr("-5").unwrap(), -5.0);
        assert_eq!(eval_expr("-(-5)").unwrap(), 5.0);
        assert_eq!(eval_expr("-(-(-5))").unwrap(), -5.0);
        
        // 一元负号与二元运算符
        assert_eq!(eval_expr("3 + -5").unwrap(), -2.0);
        assert_eq!(eval_expr("3 * -5").unwrap(), -15.0);
        
        // 一元负号与括号
        assert_eq!(eval_expr("-(3 + 5)").unwrap(), -8.0);
        assert_eq!(eval_expr("-(3 * 5)").unwrap(), -15.0);
        assert_eq!(eval_expr("-(-(3 + 5))").unwrap(), 8.0);
        
        // 复杂表达式
        assert_eq!(eval_expr("-(3 + 5) * -2").unwrap(), 16.0);
        assert_eq!(eval_expr("3 * -(5 + 2)").unwrap(), -21.0);
        assert_eq!(eval_expr("-(-3 * 4) + -(10 / 2)").unwrap(), 7.0);
    }
    
    #[test]
    fn test_unary_minus_errors() {
        // 一元负号后无操作数
        assert!(eval_expr("-").is_err());
        assert!(eval_expr("3 + -").is_err());
        assert!(eval_expr("-( )").is_err());
        
        // 一元负号位置错误
        assert!(eval_expr("3 -").is_err());
    }
}