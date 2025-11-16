use resolume_isf_shaders_rust_ffgl::isf_auto_converter::*;

fn main() {
    let mut converter = IsfAutoConverter::new();
    
    let basic_isf = r#"
    /*{
        "NAME": "Basic Color",
        "DESCRIPTION": "Simple color shader",
        "INPUTS": [
            {"NAME": "brightness", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 2.0}
        ]
    }*/
    
    void main() {
        vec2 uv = isf_FragNormCoord;
        float time = TIME * brightness;
        vec3 color = vec3(sin(time + uv.x * 10.0), cos(time + uv.y * 10.0), 0.5);
        gl_FragColor = vec4(color, 1.0);
    }
    "#;
    
    match converter.convert_to_wgsl_advanced(basic_isf) {
        Ok(result) => {
            println!("=== GENERATED WGSL ===");
            println!("{}", result.wgsl_code);
            println!("\n=== EXPECTED PATTERNS ===");
            println!("Looking for: uniforms.brightness");
            println!("Looking for: isf_FragNormCoord");
            println!("Looking for: TIME");
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}