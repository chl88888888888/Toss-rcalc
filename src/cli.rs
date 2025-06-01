use crate::evaluator::evaluate;
use crate::parser::Lexer;
use crate::history::{HistoryManager, HistoryEntry, current_timestamp};
use std::io::{self, Write};

/// 运行交互式计算器
pub fn run() -> Result<(), String> {
    println!("Welcome to the Rust Math Calculator");
    println!("Supported operators: +, -, *, /, ( )");
    println!("Type 'help' for help, 'exit' to exit the program");
    
    // 创建历史管理器
    let history_manager = HistoryManager::new("history/history.json", 50);
    
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
            println!("Thank you for using and goodbye");
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
            show_history(&history_manager);
            continue;
        }
        
        // 处理清空历史命令
        if input.eq_ignore_ascii_case("clearhistory") {
            history_manager.clear_history()
                .map_err(|e| format!("Failed to clear history: {}", e))?;
            println!("History cleared");
            continue;
        }
        
        // 执行计算
        match calculate(input) {
            Ok(result) => {
                println!(" = {}", result);
                
                // 保存成功的历史记录
                let entry = HistoryEntry {
                    expression: input.to_string(),
                    result,
                    timestamp: current_timestamp(),
                };
                
                if let Err(e) = history_manager.add_entry(entry) {
                    eprintln!("Warning: Failed to save history: {}", e);
                }
            }
            Err(e) => {
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
    println!("\nUsage:");
    println!("  Enter a mathematical expression to calculate, e.g., 3+5*2");
    println!("  Decimals are supported: 3.14, 0.5");
    println!("  Spaces are supported: 10 + 5 * 2");
    println!("  Parentheses are supported: (3+5)*2");
    println!("  Minus are supported: -5 + 3");
    println!("\nCommands:");
    println!("  help         - Displays help information");
    println!("  clear        - Clear the screen");
    println!("  history      - Display history");
    println!("  clearhistory - Clear history");
    println!("  exit         - Exit the program");
    println!("\nNotes:");
    println!("  * The divisor cannot be 0 in a division operation");
    println!("  * Function customization is not supported at this time");
}

/// 清屏
fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

/// 显示历史记录
fn show_history(manager: &HistoryManager) {
    match manager.get_history() {
        Ok(history) => {
            if history.is_empty() {
                println!("No history");
                return;
            }
            
            println!("History:");
            for (i, entry) in history.iter().rev().enumerate() {
                println!("{:2}. {} = {} [{}]", 
                    i + 1, 
                    entry.expression, 
                    entry.result,
                    entry.timestamp);
            }
        }
        Err(e) => {
            println!("Failed to load history: {}", e);
        }
    }
}