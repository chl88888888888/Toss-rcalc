mod cli;
mod evaluator;
mod history;
mod parser; // 新增历史模块
use cli::run;
fn main() {
    if let Err(e) = run() {
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
