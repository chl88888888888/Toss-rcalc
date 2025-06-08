use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tokio::fs;

const FUNC_FILE: &str = "functions/functions.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomFunction {
    pub parameters: Vec<String>,
    pub expression: String,
}

lazy_static! {
    static ref CUSTOM_FUNCTIONS: Mutex<HashMap<String, CustomFunction>> =
        Mutex::new(HashMap::new());
}

pub async fn load_functions_async() {
    if !Path::new(FUNC_FILE).exists() {
        if let Some(parent) = Path::new(FUNC_FILE).parent() {
            let _ = fs::create_dir_all(parent).await;
        }
        return;
    }
    let data = fs::read_to_string(FUNC_FILE).await.unwrap_or_default();
    let map: HashMap<String, CustomFunction> = serde_json::from_str(&data).unwrap_or_default();
    let mut global_map = CUSTOM_FUNCTIONS.lock().unwrap();
    *global_map = map;
}

pub async fn save_functions_async() {
    let json = {
        let map = CUSTOM_FUNCTIONS.lock().unwrap();
        serde_json::to_string_pretty(&*map).unwrap()
    };
    if let Some(parent) = Path::new(FUNC_FILE).parent() {
        let _ = fs::create_dir_all(parent).await;
    }
    let _ = fs::write(FUNC_FILE, json).await;
}

pub async fn register_custom_function_async(
    name: &str,
    parameters: Vec<&str>,
    expression: &str,
) -> Result<(), String> {
    {
        let mut map = CUSTOM_FUNCTIONS.lock().unwrap();
        if map.contains_key(name) {
            return Err(format!("Function {} already exists", name));
        }
        map.insert(
            name.to_string(),
            CustomFunction {
                parameters: parameters.iter().map(|s| s.to_string()).collect(),
                expression: expression.to_string(),
            },
        );
    }
    save_functions_async().await;
    Ok(())
}

pub fn expand_custom_functions(expr: &str) -> Result<String, String> {
    let mut result = expr.to_string();
    let re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^()]*)\)").unwrap();
    for _ in 0..20 {
        let map = CUSTOM_FUNCTIONS.lock().unwrap();
        let temp = re
            .replace_all(&result, |caps: &regex::Captures| {
                let name = &caps[1];
                let args_str = &caps[2];
                if let Some(func) = map.get(name) {
                    let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();
                    if args.len() != func.parameters.len() {
                        return format!(
                            "The number of function parameters is incorrect"
                        );
                    }
                    let mut body = func.expression.clone();
                    for (param, value) in func.parameters.iter().zip(args.iter()) {
                        let param_re =
                            Regex::new(&format!(r"\b{}\b", regex::escape(param))).unwrap();
                        body = param_re
                            .replace_all(&body, format!("({})", value))
                            .to_string();
                    }
                    format!("({})", body)
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();
        if temp == result {
            break;
        }
        result = temp;
    }
    if result.contains(
        "The number of function parameters is incorrect",
    ) {
        return Err("Custom function argument count mismatch".to_string());
    }
    Ok(result)
}

pub fn calculate_with_custom(expr: &str) -> Result<f64, String> {
    let expanded = expand_custom_functions(expr)?;
    let mut lexer = crate::parser::Lexer::new(&expanded);
    let tokens = lexer.tokenize()?;
    crate::evaluator::evaluate(&tokens)
}

pub fn list_custom_functions() -> Vec<(String, CustomFunction)> {
    let map = CUSTOM_FUNCTIONS.lock().unwrap();
    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

pub fn is_function_defined(name: &str) -> bool {
    let map = CUSTOM_FUNCTIONS.lock().unwrap();
    map.contains_key(name)
}
