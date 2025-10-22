pub fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    
    // Expand tilde
    if result.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            result = result.replacen("~", &home.display().to_string(), 1);
        }
    }
    
    // Handle ${VAR} syntax
    while let Some(start) = result.find("${") {
        if let Some(end) = result[start..].find('}') {
            let var_name = &result[start + 2..start + end];
            if let Ok(value) = std::env::var(var_name) {
                result.replace_range(start..start + end + 1, &value);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    // Handle $VAR syntax (without braces)
    let mut idx = 0;
    while idx < result.len() {
        if result[idx..].starts_with('$') {
            let var_start = idx + 1;
            let var_end = result[var_start..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|i| var_start + i)
                .unwrap_or(result.len());
            
            if var_end > var_start {
                let var_name = &result[var_start..var_end];
                if let Ok(value) = std::env::var(var_name) {
                    result.replace_range(idx..var_end, &value);
                    idx += value.len();
                    continue;
                }
            }
        }
        idx += 1;
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let input = "~/Documents/file.txt";
        let result = expand_env_vars(input);
        assert!(!result.starts_with("~"));
    }

    #[test]
    fn test_expand_dollar_braces() {
        std::env::set_var("TEST_VAR", "test_value");
        let input = "${TEST_VAR}/path";
        let result = expand_env_vars(input);
        assert_eq!(result, "test_value/path");
    }

    #[test]
    fn test_expand_dollar_simple() {
        std::env::set_var("USER", "testuser");
        let input = "$USER/config";
        let result = expand_env_vars(input);
        assert!(result.contains("testuser"));
    }
}
