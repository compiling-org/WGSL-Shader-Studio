use wgsl_shader_studio::isf_auto_converter::*;

fn main() {
    let converter = IsfAutoConverter::new();
    
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
        vec3 color = vec3(uv.x, uv.y, 0.5) * brightness;
        gl_FragColor = vec4(color, 1.0);
    }
    "#;
    
    match converter.convert_isf_to_wgsl(basic_isf, None) {
        Ok(result) => {
            println!("=== CONVERTED WGSL ===");
            println!("{}", result.wgsl_code);
            println!("=== END WGSL ===");
        }
        Err(e) => {
            println!("Conversion failed: {}", e);
        }
    }
}