use crate::parser::Token;

pub struct Evaluator;

impl Evaluator {
    pub fn new() -> Self {
        Evaluator
    }

    pub fn evaluate(&mut self, tokens: &[Token]) -> Result<f64, String> {
        if tokens.is_empty() {
            return Err("Empty expression".to_string());
        }

        let mut values: Vec<f64> = Vec::new();
        let mut ops: Vec<Token> = Vec::new();

        for token in tokens {
            match token {
                Token::Number(n) => values.push(*n),
                Token::FunctionCall(name, args) => {
                    let result = self.evaluate_function_call(name, args)?;
                    values.push(result);
                }
                Token::LeftParen => ops.push(token.clone()),
                Token::RightParen => {
                    while let Some(op) = ops.last() {
                        if *op == Token::LeftParen {
                            break;
                        }
                        self.perform_operation(&mut values, &mut ops)?;
                    }

                    ops.pop().ok_or("Mismatched parentheses".to_string())?;

                    if let Some(Token::UnaryMinus) = ops.last() {
                        self.perform_operation(&mut values, &mut ops)?;
                    }
                }
                Token::UnaryMinus => {
                    ops.push(token.clone());
                }
                Token::Add | Token::Subtract => {
                    while let Some(op) = ops.last() {
                        if matches!(
                            op,
                            Token::UnaryMinus
                                | Token::Multiply
                                | Token::Divide
                                | Token::Modulo
                                | Token::Power
                                | Token::Add
                                | Token::Subtract
                        ) {
                            self.perform_operation(&mut values, &mut ops)?;
                        } else {
                            break;
                        }
                    }
                    ops.push(token.clone());
                }
                Token::Multiply | Token::Divide | Token::Modulo => {
                    while let Some(op) = ops.last() {
                        if matches!(
                            op,
                            Token::Multiply | Token::Divide | Token::Modulo | Token::Power
                        ) {
                            self.perform_operation(&mut values, &mut ops)?;
                        } else {
                            break;
                        }
                    }
                    ops.push(token.clone());
                }
                Token::Power => {
                    ops.push(token.clone());
                }
                #[allow(unreachable_patterns)]
                _ => return Err(format!("Unexpected token: {:?}", token)),
            }
        }
        while let Some(op) = ops.pop() {
            match op {
                Token::UnaryMinus => {
                    if values.is_empty() {
                        return Err("Missing operand for unary minus".to_string());
                    }
                    let value = values.pop().unwrap();
                    values.push(-value);
                }
                _ => {
                    if values.len() < 2 {
                        return Err("Missing operand".to_string());
                    }
                    let b = values.pop().unwrap();
                    let a = values.pop().unwrap();
                    let res = self.apply_operator(&op, a, b)?;
                    values.push(res);
                }
            }
        }

        match values.len() {
            1 => Ok(values[0]),
            0 => Err("No result produced".to_string()),
            _ => Err(format!("Too many values in the stack: {:?}", values)),
        }
    }

    fn zero_if_tiny(val: f64) -> f64 {
        if val.abs() < 1e-8 { 0.0 } else { val }
    }

    fn evaluate_function_call(&mut self, name: &str, args: &[Token]) -> Result<f64, String> {
        match name.to_lowercase().as_str() {
            "sin" => {
                if args.len() != 1 {
                    return Err("sin() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => Ok(Self::zero_if_tiny(n.sin())),
                    _ => Err("sin() expects a number".to_string()),
                };
            }
            "cos" => {
                if args.len() != 1 {
                    return Err("cos() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => Ok(Self::zero_if_tiny(n.cos())),
                    _ => Err("cos() expects a number".to_string()),
                };
            }
            "tan" => {
                if args.len() != 1 {
                    return Err("tan() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => Ok(Self::zero_if_tiny(n.tan())),
                    _ => Err("tan() expects a number".to_string()),
                };
            }
            "log" => {
                if args.len() != 1 {
                    return Err("log() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => {
                        if n <= 0.0 {
                            Err("log() argument must be positive".to_string())
                        } else {
                            Ok(Self::zero_if_tiny(n.ln()))
                        }
                    }
                    _ => Err("log() expects a number".to_string()),
                };
            }
            "exp" => {
                if args.len() != 1 {
                    return Err("exp() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => Ok(Self::zero_if_tiny(n.exp())),
                    _ => Err("exp() expects a number".to_string()),
                };
            }
            "arcsin" => {
                if args.len() != 1 {
                    return Err("asin() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => Ok(Self::zero_if_tiny(n.asin())),
                    _ => Err("asin() expects a number".to_string()),
                };
            }
            "arccos" => {
                if args.len() != 1 {
                    return Err("acos() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => Ok(Self::zero_if_tiny(n.acos())),
                    _ => Err("acos() expects a number".to_string()),
                };
            }
            "arctan" => {
                if args.len() != 1 {
                    return Err("atan() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => Ok(Self::zero_if_tiny(n.atan())),
                    _ => Err("atan() expects a number".to_string()),
                };
            }
            "fact" | "factorial" => {
                if args.len() != 1 {
                    return Err("fact() expects 1 argument".to_string());
                }
                return match args[0] {
                    Token::Number(n) => {
                        if n < 0.0 || n.fract() != 0.0 {
                            Err("fact() expects a non-negative integer".to_string())
                        } else {
                            let mut res = 1u128;
                            let mut i = 1u128;
                            let n = n as u128;
                            while i <= n {
                                res = res.checked_mul(i).ok_or("fact() overflow")?;
                                i += 1;
                            }
                            Ok(res as f64)
                        }
                    }
                    _ => Err("fact() expects a number".to_string()),
                };
            }
            "comb" => {
                if args.len() != 2 {
                    return Err("comb() expects 2 arguments".to_string());
                }
                return match (&args[0], &args[1]) {
                    (Token::Number(n), Token::Number(k)) => {
                        if *n < 0.0 || *k < 0.0 || n.fract() != 0.0 || k.fract() != 0.0 || k > n {
                            Err("comb(n, k) expects 0 <= k <= n, both integers".to_string())
                        } else {
                            let n = *n as u128;
                            let k = *k as u128;
                            let mut res = 1u128;
                            for i in 0..k {
                                res = res.checked_mul(n - i).ok_or("comb() overflow")?;
                                res = res.checked_div(i + 1).ok_or("comb() division by zero")?;
                            }
                            Ok(res as f64)
                        }
                    }
                    _ => Err("comb() expects two numbers".to_string()),
                };
            }
            "perm" => {
                if args.len() != 2 {
                    return Err("perm() expects 2 arguments".to_string());
                }
                return match (&args[0], &args[1]) {
                    (Token::Number(n), Token::Number(k)) => {
                        if *n < 0.0 || *k < 0.0 || n.fract() != 0.0 || k.fract() != 0.0 || k > n {
                            Err("perm(n, k) expects 0 <= k <= n, both integers".to_string())
                        } else {
                            let n = *n as u128;
                            let k = *k as u128;
                            let mut res = 1u128;
                            for i in 0..k {
                                res = res.checked_mul(n - i).ok_or("perm() overflow")?;
                            }
                            Ok(res as f64)
                        }
                    }
                    _ => Err("perm() expects two numbers".to_string()),
                };
            }
            _ => {}
        }

        if !crate::functions::is_function_defined(name) {
            return Err(format!("Function '{}' is not defined", name));
        }
        let arg_strs: Vec<String> = args
            .iter()
            .map(|arg| match arg {
                Token::Number(n) => Ok(n.to_string()),
                _ => Err("Function arguments must be numbers".to_string()),
            })
            .collect::<Result<Vec<String>, String>>()?;

        let expr = format!("{}({})", name, arg_strs.join(","));
        crate::functions::calculate_with_custom(&expr)
    }

    fn apply_operator(&self, op: &Token, a: f64, b: f64) -> Result<f64, String> {
        match op {
            Token::Add => Ok(a + b),
            Token::Subtract => Ok(a - b),
            Token::Multiply => Ok(a * b),
            Token::Divide => {
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(a / b)
            }
            Token::Modulo => {
                if a.fract() != 0.0 || b.fract() != 0.0 {
                    return Err("Modulo operation requires integer operands".to_string());
                }
                if b == 0.0 {
                    return Err("Modulo by zero".to_string());
                }
                Ok((a as i64 % b as i64) as f64)
            }
            Token::Power => {
                if a == 0.0 && b == 0.0 {
                    return Err("Undefined operation: 0^0".to_string());
                }
                if a < 0.0 && b.fract() != 0.0 {
                    return Err("Negative base with fractional exponent is undefined".to_string());
                }
                let result = a.powf(b);
                if result.is_nan() {
                    return Err(format!("Invalid operation: ({})^({})", a, b));
                }
                Ok(result)
            }
            _ => Err(format!("Unexpected operator: {:?}", op)),
        }
    }

    fn perform_operation(&self, values: &mut Vec<f64>, ops: &mut Vec<Token>) -> Result<(), String> {
        let op = ops.pop().ok_or("Missing operator".to_string())?;
        if op == Token::UnaryMinus {
            if values.is_empty() {
                return Err("Missing operand for unary minus".to_string());
            }
            let value = values.pop().unwrap();
            values.push(-value);
            return Ok(());
        }
        if values.len() < 2 {
            return Err("Missing operand".to_string());
        }
        let b = values.pop().unwrap();
        let a = values.pop().unwrap();

        let res = self.apply_operator(&op, a, b)?;
        values.push(res);
        Ok(())
    }
}

pub fn evaluate(tokens: &[Token]) -> Result<f64, String> {
    Evaluator::new().evaluate(tokens)
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
        assert_eq!(eval_expr("-(-(-(-5)))").unwrap(), 5.0);

        //连续一元负号
        assert_eq!(eval_expr("--5").unwrap(), 5.0);
        assert_eq!(eval_expr("---5").unwrap(), -5.0);
        assert_eq!(eval_expr("----5").unwrap(), 5.0);

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
        assert_eq!(eval_expr("-(3 * -(5 + 2))").unwrap(), 21.0);
        assert_eq!(eval_expr("-(-2 ^ 3)").unwrap(), 8.0);
        assert_eq!(eval_expr("-(3 + -(-5))").unwrap(), -8.0);
    }

    #[test]
    fn test_complex_expression() {
        assert_eq!(eval_expr("-(-3 * 4) + -(10 / 2)").unwrap(), 7.0);
        assert_eq!(eval_expr("-(-3 * 4) * -(10 / 2)").unwrap(), -60.0);
        assert_eq!(eval_expr("-(-3 * -4) + -(10 / 2)").unwrap(), -17.0);
        assert_eq!(eval_expr("-(2 * 3) + -(-4 / 2)").unwrap(), -4.0);
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

    #[test]
    fn test_unary_minus_priority() {
        // 一元负号优先级测试
        assert_eq!(eval_expr("-2+4").unwrap(), 2.0); // (-2) + 4 = 2
        assert_eq!(eval_expr("-2-4").unwrap(), -6.0); // (-2) - 4 = -6
        assert_eq!(eval_expr("2+-4").unwrap(), -2.0); // 2 + (-4) = -2
        assert_eq!(eval_expr("2--4").unwrap(), 6.0); // 2 - (-4) = 6
        assert_eq!(eval_expr("-2*3").unwrap(), -6.0); // (-2) * 3 = -6
        assert_eq!(eval_expr("-2/4").unwrap(), -0.5); // (-2) / 4 = -0.5
        assert_eq!(eval_expr("2*-4").unwrap(), -8.0); // 2 * (-4) = -8
        assert_eq!(eval_expr("2/-4").unwrap(), -0.5); // 2 / (-4) = -0.5
        assert_eq!(eval_expr("-2^3").unwrap(), -8.0); // -(2^3) = -8
        assert_eq!(eval_expr("(-2)^3").unwrap(), -8.0); // (-2)^3 = -8
        assert_eq!(eval_expr("(-2)^2").unwrap(), 4.0); // (-2)^2 = 4

        // 复杂表达式
        assert_eq!(eval_expr("-3*4+5").unwrap(), -7.0); // (-3*4)+5 = -12+5 = -7
        assert_eq!(eval_expr("3*-4+5").unwrap(), -7.0); // 3*(-4)+5 = -12+5 = -7
        assert_eq!(eval_expr("3+4*-5").unwrap(), -17.0); // 3+4*(-5) = 3-20 = -17
        assert_eq!(eval_expr("(3+4)*-5").unwrap(), -35.0); // (3+4)*(-5) = 7*-5 = -35
        assert_eq!(eval_expr("-3+4*5").unwrap(), 17.0); // (-3)+4*5 = -3+20 = 17
        assert_eq!(eval_expr("3+-4*5").unwrap(), -17.0); // 3+(-4*5) = 3-20 = -17
    }

    #[test]
    fn test_modulo_operations() {
        // 整数取模运算
        assert_eq!(eval_expr("10 % 3").unwrap(), 1.0);
        assert_eq!(eval_expr("15 % 4").unwrap(), 3.0);

        // 负数取模
        assert_eq!(eval_expr("-10 % 3").unwrap(), -1.0);
        assert_eq!(eval_expr("10 % -3").unwrap(), 1.0);
        assert_eq!(eval_expr("-10 % -3").unwrap(), -1.0);

        // 优先级测试
        assert_eq!(eval_expr("10 + 8 % 3").unwrap(), 12.0); // 8%3=2, 10+2=12
        assert_eq!(eval_expr("10 * 8 % 3").unwrap(), 2.0); // 10*8=80, 80%3=2
        assert_eq!(eval_expr("(10 + 8) % 3").unwrap(), 0.0); // 18%3=0

        // 除零错误
        assert!(eval_expr("10 % 0").is_err());

        // 浮点数取模 - 应该报错
        assert!(eval_expr("7.5 % 3.2").is_err());
        assert!(eval_expr("10.5 % 3.5").is_err());
    }

    #[test]
    fn test_mixed_operations() {
        // 混合运算
        assert_eq!(eval_expr("2 ^ 3 + 10 % 3").unwrap(), 9.0); // 8 + 1 = 9
        assert_eq!(eval_expr("(5 + 3) % 4 * 2 ^ 2").unwrap(), 0.0); // 8%4=0, 0*4=0
        assert_eq!(eval_expr("10 % 3 ^ 2").unwrap(), 1.0); // 3^2=9, 10%9=1
        assert_eq!(eval_expr("2 ^ (3 % 2)").unwrap(), 2.0); // 3%2=1, 2^1=2

        // 浮点数取模在混合表达式中
        assert!(eval_expr("10.5 % 3 + 2").is_err());
        assert!(eval_expr("2 * (10 % 3.5)").is_err());
    }

    #[test]
    fn test_power_operations() {
        // 基本幂运算
        assert_eq!(eval_expr("2 ^ 3").unwrap(), 8.0);
        assert_eq!(eval_expr("3 ^ 2").unwrap(), 9.0);
        assert_eq!(eval_expr("4 ^ 0.5").unwrap(), 2.0); // 平方根

        // 负数幂运算
        assert_eq!(eval_expr("2 ^ -2").unwrap(), 0.25);
        assert_eq!(eval_expr("-2 ^ 3").unwrap(), -8.0);
        assert_eq!(eval_expr("(-2) ^ 3").unwrap(), -8.0);
        assert_eq!(eval_expr("(-2) ^ 2").unwrap(), 4.0);

        // 优先级测试
        assert_eq!(eval_expr("2 * 3 ^ 2").unwrap(), 18.0); // 3^2=9, 2*9=18
        assert_eq!(eval_expr("(2 * 3) ^ 2").unwrap(), 36.0); // 6^2=36
        assert_eq!(eval_expr("2 ^ 3 ^ 2").unwrap(), 512.0); // 2^(3^2)=2^9=512 (右结合)
        assert_eq!(eval_expr("4 ^ -0.5").unwrap(), 0.5); // 1/sqrt(4)=0.5

        // 特殊值
        assert_eq!(eval_expr("0 ^ 5").unwrap(), 0.0);
        assert_eq!(eval_expr("5 ^ 0").unwrap(), 1.0);

        // 错误情况
        assert!(eval_expr("0 ^ 0").is_err()); // 0^0未定义
        assert!(eval_expr("(-2) ^ 0.5").is_err()); // 负数平方根
    }

    #[test]
    fn test_nan_handling() {
        // 检查NaN处理
        assert!(eval_expr("(-2) ^ 0.5").is_err());
        assert!(eval_expr("(-1) ^ 0.5").is_err());
        assert!(eval_expr("(-4) ^ (1/2)").is_err());
        assert!(eval_expr("(-8) ^ (1/3)").is_err());

        // 有效操作
        assert_eq!(eval_expr("(-8) ^ (1/1)").unwrap(), -8.0);
        assert_eq!(eval_expr("(-8) ^ 1").unwrap(), -8.0);
        assert_eq!(eval_expr("(-8) ^ 2").unwrap(), 64.0);
        assert_eq!(eval_expr("(-8) ^ -1").unwrap(), -0.125);
    }

    #[test]
    fn test_power_mixed_operations() {
        // 混合运算
        assert_eq!(eval_expr("2 ^ 3 + 10 % 3").unwrap(), 9.0); // 8 + 1 = 9
        assert_eq!(eval_expr("(5 + 3) % 4 * 2 ^ 2").unwrap(), 0.0); // 8%4=0, 0*4=0
        assert_eq!(eval_expr("10 % 3 ^ 2").unwrap(), 1.0); // 3^2=9, 10%9=1
        assert_eq!(eval_expr("2 ^ (3 % 2)").unwrap(), 2.0); // 3%2=1, 2^1=2
    }

    #[test]
    fn test_power_right_associativity() {
        // 右结合性测试
        assert_eq!(eval_expr("2 ^ 3 ^ 2").unwrap(), 512.0); // 2^(3^2)=512
        assert_eq!(eval_expr("2 ^ (3 ^ 2)").unwrap(), 512.0);
        assert_eq!(eval_expr("(2 ^ 3) ^ 2").unwrap(), 64.0);
        assert_eq!(eval_expr("3 ^ 2 ^ 2").unwrap(), 81.0); // 3^(2^2)=3^4=81
        assert_eq!(eval_expr("4 ^ 3 ^ 2").unwrap(), 262144.0); // 4^(3^2)=4^9=262144
        assert_eq!(eval_expr("2 ^ 3 ^ 4").unwrap(), 2417851639229258349412352.0); // 2^(3^4)=2^81
    }
}
