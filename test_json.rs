fn main() {
    let json_str = r#"{
                "NAME": "Basic Color",
                "DESCRIPTION": "Simple color shader",
                "INPUTS": [
                    {"NAME": "brightness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0}
                ]
            }"#;
    
    println!("Testing JSON:");
    println!("{}", json_str);
    
    match json_str.parse::<serde_json::Value>() {
        Ok(parsed) => println!("Parsed successfully: {:?}", parsed),
        Err(e) => println!("Parse error: {}", e),
    }
}