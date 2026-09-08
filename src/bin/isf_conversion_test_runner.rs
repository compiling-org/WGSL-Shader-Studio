//! ISF Conversion Test Runner
//! Comprehensive testing of ISF to WGSL conversion system

#[cfg(not(feature = "makepad_ui"))]
use resolume_isf_shaders_rust_ffgl::isf_auto_converter::IsfAutoConverter;

#[cfg(not(feature = "makepad_ui"))]
fn main() {
    println!("🚀 ISF Conversion Test Runner");
    println!("================================");

    // Initialize the converter and tester
    let mut converter = IsfAutoConverter::new();

    println!("📋 Running comprehensive ISF conversion tests...");

    // Test basic shader conversion
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
            println!("✅ Basic conversion passed");
            println!("Generated WGSL:\n{}", result.wgsl_code);
        }
        Err(e) => {
            println!("❌ Conversion failed: {}", e);
        }
    }

    println!("\n🎉 ISF conversion test runner completed!");
}

#[cfg(feature = "makepad_ui")]
fn main() {
    println!("isf-conversion-test-runner binary is not available in Makepad UI mode");
}