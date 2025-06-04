mod cli;
mod evaluator;
mod history;
mod parser;

use clap::Parser;
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
#[command(name = "rcalc", version = "0.1.0", about = "Rust calculator")]
struct Cli {
    expression: Option<String>,

    ///Print history
    #[arg(short = 'H', long)]
    history: bool,

    ///Clear history
    #[arg(short = 'C', long)]
    clear_history: bool,

    ///Interaction mode
    #[arg(short = 'i', long)]
    interactive: bool,

    ///Silent mode (Print result only)
    #[arg(short = 'q', long)]
    quiet: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let history_manager = history::HistoryManager::new("history/calc_history.json", 100);

    if cli.clear_history {
        if let Err(e) = history_manager.clear_history().await {
            eprintln!("Failed to clear history: {}", e);
        } else {
            println!("Clear the history");
        }
        return;
    }

    if cli.history {
        cli::show_history(&history_manager).await;
        return;
    }

    if let Some(expr) = cli.expression {
        if let Ok(result) = cli::calculate(&expr) {
            if !cli.quiet {
                println!("{} = {}", expr, result);
            } else {
                println!("{}", result);
            }

            let entry = history::HistoryEntry {
                expression: expr.clone(),
                result,
                timestamp: history::current_timestamp(),
            };
            if let Err(e) = history_manager.add_entry(entry).await {
                eprintln!("Warning: Failed to save history {}", e);
            }
        } else {
            eprintln!("Failed to solve the expression :'{}' ", expr);
        }
        return;
    }

    if !atty::is(atty::Stream::Stdin) {
        let stdin = io::stdin();
        let mut quiet = cli.quiet;

        for line in stdin.lock().lines() {
            let expr = line.unwrap_or_default().trim().to_string();
            if expr.is_empty() {
                continue;
            }

            if let Ok(result) = cli::calculate(&expr) {
                if !quiet {
                    println!("{} = {}", expr, result);
                } else {
                    println!("{}", result);
                }

                let entry = history::HistoryEntry {
                    expression: expr.clone(),
                    result,
                    timestamp: history::current_timestamp(),
                };
                if let Err(e) = history_manager.add_entry(entry).await {
                    eprintln!("Warning: Failed to save history :{}", e);
                }
            } else {
                eprintln!("Failed to solve the expression:'{}' ", expr);
            }
            quiet = true;
        }
        return;
    }

    if let Err(e) = cli::run(&history_manager).await {
        eprintln!("Program error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::{evaluator::evaluate, parser::Lexer};
    #[test]
    fn test_integration() {
        let input = "3+5*2-8/4";
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(evaluate(&tokens).unwrap(), 11.0);
    }
}
