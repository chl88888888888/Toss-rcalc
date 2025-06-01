use crate::{evaluator::evaluate, parser::Lexer};
use std::io::{self, Write};

/// 运行交互式计算器
pub fn run() -> Result<(), String> {
    println!("Welcome to the Rust Math Calculator");
    println!("Supported operators: , -, *, /, ()");
    println!("Type 'help' to see help, 'exit' to exit the program");
    
    // 历史记录: 存储 (表达式, 结果) 元组
    let mut history: Vec<(String, f64)> = Vec::new();
    
    loop {
        // 显示提示符
        print!("> ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        
        // 读取用户输入
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
        // 清理输入
        let input = input.trim();
        
        // 处理退出命令
        if input.eq_ignore_ascii_case("exit") || input.is_empty() {
            println!("Thanks for using and goodbye");
            return Ok(());
        }
        
        // 处理帮助命令
        if input.eq_ignore_ascii_case("help") {
            show_help();
            continue;
        }
        
        // 处理清屏命令
        if input.eq_ignore_ascii_case("clear") {
            clear_screen();
            continue;
        }
        
        // 处理历史记录命令
        if input.eq_ignore_ascii_case("history") {
            show_history(&history);
            continue;
        }
        
        // 执行计算
        match calculate(input) {
            Ok(result) => {
                // 打印结果
                println!(" = {}", result);
                
                // 保存成功的历史记录 (表达式, 结果)
                history.push((input.to_string(), result));
                
                // 限制历史记录大小
                if history.len() > 20 {
                    history.remove(0); // 只保留最近20条历史
                }
            }
            Err(e) => {
                // 出错时不保存历史记录
                println!("Error: {}", e);
            }
        }
    }
}

/// 计算表达式
fn calculate(input: &str) -> Result<f64, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    evaluate(&tokens)
}

/// 显示帮助信息
fn show_help() {
    println!("\nDirections for use:");
    println!("  Enter a mathematical expression to calculate, for example: 3 5*2");
    println!("  Decimals are supported: 3.14, 0.5");
    println!("  Spaces are supported: 10 + 5 * 2");
    println!("  Parentheses are supported: ( 10 + 5 ) * ( 6 )");
    println!("  Minus are supported: - 5 * ( - 6 - 1 )");
    println!("\nCLI:");
    println!("  help    - Displays help information");
    println!("  clear   - Clear the screen");
    println!("  history - Display history");
    println!("  exit    - Exit the program");
    println!("\nThings for attention:");
    println!("  * The divisor cannot be 0 in a division operation");
    println!("  * Function customization is not supported at this time");
}

/// 清屏
fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

/// 显示历史记录
fn show_history(history: &[(String, f64)]) {
    if history.is_empty() {
        println!("No history");
        return;
    }
    
    println!("History:");
    for (i, (expr, result)) in history.iter().enumerate() {
        println!("{:2}. {} = {}", i + 1, expr, result);
    }
}