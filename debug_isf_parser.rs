fn main() {
    let test_isf = r#"
            /*{
                "NAME": "Basic Color",
                "DESCRIPTION": "Simple color shader",
                "INPUTS": [
                    {"NAME": "brightness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0}
                ]
            }*/
            
            void main() {
                vec2 uv = isf_FragNormCoord;
                vec3 color = vec3(uv.x, uv.y, 0.5) * brightness;
                gl_FragColor = vec4(color, 1.0);
            }
            "#;
    
    println!("Original ISF code:");
    println!("{}", test_isf);
    println!("\n{}", "=".repeat(50));
    
    // Find JSON boundaries
    if let Some(json_start) = test_isf.find("/*{") {
        println!("Found JSON start at position: {}", json_start);
        
        if let Some(json_end) = test_isf.find("}*/") {
            println!("Found JSON end at position: {}", json_end);
            
            let json_str = &test_isf[json_start + 2..json_end];
            println!("\nExtracted JSON:");
            println!("{}", json_str);
            println!("\nJSON length: {}", json_str.len());
            
            // Try to parse
            match json_str.parse::<serde_json::Value>() {
                Ok(parsed) => println!("\nParsed successfully: {:?}", parsed),
                Err(e) => println!("\nParse error: {}", e),
            }
        }
    }
}