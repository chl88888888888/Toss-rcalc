use crate::evaluator::evaluate;
use crate::parser::Lexer;
use crate::history::{HistoryEntry, HistoryManager};
use std::io::{self, Write};

/// run rust calculator (asynchronous)
pub async fn run(history_manager: &HistoryManager) -> Result<(), String> {
    println!("Welcome to the Rust Math Calculator");
    println!("Supported operators: +, -, *, /, ( ), %, ^");
    println!("Type 'help' for help, 'exit' to exit the program");
    loop {
        // show ">"
        print!("> ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        
        // read user's input
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        
        // clean input
        let input = input.trim();
        
        // solve "exit" command
        if input.eq_ignore_ascii_case("exit") || input.is_empty() {
            println!("Thank you for using and goodbye");
            return Ok(());
        }
        
        // solve "help" command
        if input.eq_ignore_ascii_case("help") {
            show_help();
            continue;
        }
        
        // solve "clear" command
        if input.eq_ignore_ascii_case("clear") {
            clear_screen();
            continue;
        }
        
        // solve "history" command
        if input.eq_ignore_ascii_case("history") {
            show_history(history_manager).await;
            continue;
        }
        
        // solve "clearhistory" command
        if input.eq_ignore_ascii_case("clearhistory") {
            history_manager.clear_history()
                .await
                .map_err(|e| format!("Failed to clear history: {}", e))?;
            println!("History cleared");
            continue;
        }
        
        // Perform calculations
        match calculate(input) {
            Ok(result) => {
                println!(" = {}", result);
                
                // Save a history of success (asynchronous)
                let entry = HistoryEntry {
                    expression: input.to_string(),
                    result,
                    timestamp: crate::history::current_timestamp(),
                };
                
                // Use the cloned manager
                let manager_clone = history_manager.clone_manager();
                tokio::spawn(async move {
                    if let Err(e) = manager_clone.add_entry(entry).await {
                        eprintln!("Warning: Failed to save history: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

/// Calculation Expressions (Public API)
pub fn calculate(input: &str) -> Result<f64, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    evaluate(&tokens)
}

/// show help
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
    println!("  * A negative presquare will lead to an error");
    println!("  * Power operations support right conjugation (2^3^2 = 2^(3^2) = 512)");
}

/// clear screen
fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

/// Show history (asynchronous)
pub async fn show_history(manager: &HistoryManager) {
    match manager.get_history().await {
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