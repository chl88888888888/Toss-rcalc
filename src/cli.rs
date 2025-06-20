use crate::evaluator::evaluate;
use crate::functions;
use crate::history::{HistoryEntry, HistoryManager};
use crate::parser::Lexer;
use regex::Regex;
use std::io::{self, Write};

pub async fn run(history_manager: &HistoryManager) -> Result<(), String> {
    println!("Welcome to the Rust Math Calculator");
    println!("Supported operators: +, -, *, /, ( ), %, ^");
    println!("Type 'help' for help, 'exit' to exit the program");
    loop {
        print!("> ");
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| e.to_string())?;

        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") || input.is_empty() {
            println!("Thank you for using and goodbye");
            return Ok(());
        }

        if input.eq_ignore_ascii_case("help") {
            show_help();
            continue;
        }

        if input.eq_ignore_ascii_case("clear") {
            clear_screen();
            continue;
        }

        if input.eq_ignore_ascii_case("history") {
            show_history(history_manager).await;
            continue;
        }

        if input.eq_ignore_ascii_case("clearhistory") {
            history_manager
                .clear_history()
                .await
                .map_err(|e| format!("Failed to clear history: {}", e))?;
            println!("History cleared");
            continue;
        }

        if input.starts_with("define ") {
            let def = input.strip_prefix("define ").unwrap();
            match define_function_async(def).await {
                Ok(_) => println!("Function defined successfully"),
                Err(e) => println!("Function definition failed: {}", e),
            }
            continue;
        }

        match calculate(input) {
            Ok(result) => {
                println!(" = {}", result);

                let entry = HistoryEntry {
                    expression: input.to_string(),
                    result,
                    timestamp: crate::history::current_timestamp(),
                };

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

        if input.eq_ignore_ascii_case("functions") {
            for (name, func) in functions::list_custom_functions() {
                println!(
                    "{}({}) = {}",
                    name,
                    func.parameters.join(", "),
                    func.expression
                );
            }
            continue;
        }
    }
}

pub fn calculate(input: &str) -> Result<f64, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    evaluate(&tokens)
}

fn show_help() {
    println!("\nUsage:");
    println!("  Enter a mathematical expression to calculate, e.g., 3+5*2");
    println!("  Decimals are supported: 3.14, 0.5");
    println!("  Spaces are supported: 10 + 5 * 2");
    println!("  Parentheses are supported: (3+5)*2");
    println!("  Minus are supported: -5 + 3");
    println!("  Function calls are supported: f(2), g(1, 2)");
    println!("\nCommands:");
    println!("  help         - Displays help information");
    println!("  clear        - Clear the screen");
    println!("  history      - Display history");
    println!("  clearhistory - Clear history");
    println!("  exit         - Exit the program");
    println!("\nNotes:");
    println!("  * The divisor cannot be 0 in a division operation");
    println!("  * Function customization is supported");
    println!("  * A negative base with a fractional exponent will lead to an error");
    println!("  * Power operations support right associativity (2^3^2 = 2^(3^2) = 512)");
}

fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

pub async fn show_history(manager: &HistoryManager) {
    match manager.get_history().await {
        Ok(history) => {
            if history.is_empty() {
                println!("No history");
                return;
            }

            println!("History:");
            for (i, entry) in history.iter().rev().enumerate() {
                println!(
                    "{:2}. {} = {} [{}]",
                    i + 1,
                    entry.expression,
                    entry.result,
                    entry.timestamp
                );
            }
        }
        Err(e) => {
            println!("Failed to load history: {}", e);
        }
    }
}

pub async fn define_function_async(definition: &str) -> Result<(), String> {
    let re = Regex::new(r"^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*?)\)\s*=\s*(.+)\s*$").unwrap();
    let caps = re
        .captures(definition)
        .ok_or("Function definition error, should be name(param1, param2, ...) = expression")?;
    let name = caps.get(1).unwrap().as_str().trim();
    let params_str = caps.get(2).unwrap().as_str().trim();
    let expression = caps.get(3).unwrap().as_str().trim();

    let parameters: Vec<&str> = if params_str.is_empty() {
        Vec::new()
    } else {
        params_str.split(',').map(|s| s.trim()).collect()
    };

    let mut unique_params = parameters.clone();
    unique_params.sort();
    unique_params.dedup();
    if unique_params.len() != parameters.len() {
        return Err("Parameter names must be unique".to_string());
    }

    crate::functions::register_custom_function_async(name, parameters, expression).await
}
