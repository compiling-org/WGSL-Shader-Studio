//! Shader conversion utilities for ISF to WGSL/GLSL

// Define the types locally for the binary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IsfShader {
    pub name: String,
    pub source: String,
    pub inputs: Vec<ShaderInput>,
    pub outputs: Vec<ShaderOutput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShaderInput {
    pub name: String,
    pub input_type: InputType,
    pub value: ShaderValue,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub default: Option<f32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShaderOutput {
    pub name: String,
    pub output_type: OutputType,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InputType {
    Float,
    Bool,
    Color,
    Point2D,
    Image,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ShaderValue {
    Float(f32),
    Bool(bool),
    Color([f32; 4]),
    Point2D([f32; 2]),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum OutputType {
    Image,
    Float,
}
use std::collections::HashMap;

/// Convert ISF shader to WGSL
pub fn isf_to_wgsl(shader: &IsfShader) -> Result<String, Box<dyn std::error::Error>> {
    let mut wgsl = String::new();

    // Add WGSL header
    wgsl.push_str("@group(0) @binding(0)\n");
    wgsl.push_str("var<uniform> time: f32;\n");
    wgsl.push_str("@group(0) @binding(1)\n");
    wgsl.push_str("var<uniform> resolution: vec2<f32>;\n");

    // Add input uniforms
    for input in &shader.inputs {
        match input.input_type {
            InputType::Float => {
                wgsl.push_str(&format!("@group(0) @binding({})\n", 2 + shader.inputs.iter().position(|i| i.name == input.name).unwrap()));
                wgsl.push_str(&format!("var<uniform> {}: f32;\n", input.name));
            }
            InputType::Bool => {
                wgsl.push_str(&format!("@group(0) @binding({})\n", 2 + shader.inputs.iter().position(|i| i.name == input.name).unwrap()));
                wgsl.push_str(&format!("var<uniform> {}: u32;\n", input.name));
            }
            InputType::Color => {
                wgsl.push_str(&format!("@group(0) @binding({})\n", 2 + shader.inputs.iter().position(|i| i.name == input.name).unwrap()));
                wgsl.push_str(&format!("var<uniform> {}: vec4<f32>;\n", input.name));
            }
            InputType::Point2D => {
                wgsl.push_str(&format!("@group(0) @binding({})\n", 2 + shader.inputs.iter().position(|i| i.name == input.name).unwrap()));
                wgsl.push_str(&format!("var<uniform> {}: vec2<f32>;\n", input.name));
            }
            InputType::Image => {
                wgsl.push_str(&format!("@group(1) @binding({})\n", shader.inputs.iter().position(|i| i.name == input.name).unwrap()));
                wgsl.push_str(&format!("var {}: texture_2d<f32>;\n", input.name));
                wgsl.push_str(&format!("@group(1) @binding({})\n", shader.inputs.iter().position(|i| i.name == input.name).unwrap() + 1));
                wgsl.push_str(&format!("var {}_sampler: sampler;\n", input.name));
            }
        }
    }

    // Add output texture
    wgsl.push_str("@group(2) @binding(0)\n");
    wgsl.push_str("var output_texture: texture_storage_2d<rgba8unorm, write>;\n");

    // Convert GLSL shader code to WGSL
    let glsl_code = extract_glsl_from_isf(&shader.source)?;
    let wgsl_code = glsl_to_wgsl(&glsl_code)?;

    wgsl.push_str("@compute @workgroup_size(16, 16)\n");
    wgsl.push_str("fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {\n");
    wgsl.push_str("    let coords = vec2<i32>(global_id.xy);\n");
    wgsl.push_str("    let uv = vec2<f32>(coords) / resolution;\n");

    // Add converted shader code
    wgsl.push_str(&wgsl_code);

    wgsl.push_str("}\n");

    Ok(wgsl)
}

/// Extract GLSL code from ISF shader source
fn extract_glsl_from_isf(source: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Find the GLSL code between the JSON metadata and the end
    let mut glsl_start = 0;

    // Skip JSON metadata
    if let Some(json_end) = source.find("}*/") {
        glsl_start = json_end + 3;
    }

    let glsl_code = source[glsl_start..].trim();

    // Basic GLSL to WGSL conversion
    let wgsl_code = glsl_code
        .replace("void main()", "fn main()")
        .replace("gl_FragCoord", "vec4<f32>(vec2<f32>(coords), 0.0, 1.0)")
        .replace("texture2D", "textureSample")
        .replace("vec2(", "vec2<f32>(")
        .replace("vec3(", "vec3<f32>(")
        .replace("vec4(", "vec4<f32>(")
        .replace("float(", "f32(")
        .replace("int(", "i32(")
        .replace("bool(", "bool(")
        .replace("mix(", "mix<f32>(")
        .replace("clamp(", "clamp<f32>(")
        .replace("sin(", "sin<f32>(")
        .replace("cos(", "cos<f32>(")
        .replace("tan(", "tan<f32>(")
        .replace("pow(", "pow<f32>(")
        .replace("exp(", "exp<f32>(")
        .replace("log(", "log<f32>(")
        .replace("sqrt(", "sqrt<f32>(")
        .replace("abs(", "abs<f32>(")
        .replace("floor(", "floor<f32>(")
        .replace("ceil(", "ceil<f32>(")
        .replace("fract(", "fract<f32>(")
        .replace("mod(", "f32("); // Basic mod replacement

    Ok(wgsl_code)
}

/// Convert GLSL code to WGSL
fn glsl_to_wgsl(glsl: &str) -> Result<String, Box<dyn std::error::Error>> {
    // This is a basic conversion - a full implementation would need proper AST parsing
    let mut wgsl = glsl.to_string();

    // Replace GLSL-specific constructs
    wgsl = wgsl.replace("gl_FragColor", "let color");
    wgsl = wgsl.replace("texture2D", "textureSample");

    // Add texture write at the end if it's a fragment shader
    if wgsl.contains("let color") {
        wgsl.push_str("    textureStore(output_texture, coords, color);\n");
    }

    Ok(wgsl)
}

/// Convert WGSL to GLSL for OpenGL compatibility
pub fn wgsl_to_glsl(wgsl: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut glsl = String::new();

    glsl.push_str("#version 330 core\n");
    glsl.push_str("uniform float time;\n");
    glsl.push_str("uniform vec2 resolution;\n");
    glsl.push_str("uniform vec2 mouse;\n");
    glsl.push_str("out vec4 fragColor;\n");

    // Convert WGSL uniforms to GLSL
    for line in wgsl.lines() {
        if line.contains("var<uniform>") {
            if line.contains("f32") {
                glsl.push_str(&line.replace("var<uniform>", "uniform").replace(": f32;", ";"));
            } else if line.contains("vec2<f32>") {
                glsl.push_str(&line.replace("var<uniform>", "uniform").replace(": vec2<f32>;", ";"));
            } else if line.contains("vec3<f32>") {
                glsl.push_str(&line.replace("var<uniform>", "uniform").replace(": vec3<f32>;", ";"));
            } else if line.contains("vec4<f32>") {
                glsl.push_str(&line.replace("var<uniform>", "uniform").replace(": vec4<f32>;", ";"));
            } else if line.contains("u32") {
                glsl.push_str(&line.replace("var<uniform>", "uniform").replace(": u32;", ";"));
            }
        }
    }

    glsl.push_str("void main() {\n");
    glsl.push_str("    vec2 uv = gl_FragCoord.xy / resolution;\n");

    // Convert WGSL code to GLSL
    let wgsl_body = extract_wgsl_body(wgsl)?;
    let glsl_body = wgsl_body
        .replace("vec2<f32>", "vec2")
        .replace("vec3<f32>", "vec3")
        .replace("vec4<f32>", "vec4")
        .replace("f32(", "float(")
        .replace("i32(", "int(")
        .replace("u32(", "uint(")
        .replace("bool(", "bool(")
        .replace("textureSample", "texture")
        .replace("let ", "")
        .replace(";", " = ")
        .replace("textureStore(output_texture, coords, color)", "fragColor = color")
        .replace("gl_FragCoord", "vec4(gl_FragCoord.xy, 0.0, 1.0)")
        .replace("position.xy", "gl_FragCoord.xy");

    glsl.push_str(&glsl_body);
    glsl.push_str("}\n");

    Ok(glsl)
}

/// Convert WGSL to HLSL for DirectX compatibility
pub fn wgsl_to_hlsl(wgsl: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut hlsl = String::new();

    hlsl.push_str("cbuffer Constants {\n");
    hlsl.push_str("    float time;\n");
    hlsl.push_str("    float2 resolution;\n");
    hlsl.push_str("    float2 mouse;\n");
    hlsl.push_str("};\n\n");

    // Convert WGSL uniforms to HLSL
    for line in wgsl.lines() {
        if line.contains("var<uniform>") {
            if line.contains("f32") {
                hlsl.push_str(&format!("float {};\n", extract_uniform_name(line)));
            } else if line.contains("vec2<f32>") {
                hlsl.push_str(&format!("float2 {};\n", extract_uniform_name(line)));
            } else if line.contains("vec3<f32>") {
                hlsl.push_str(&format!("float3 {};\n", extract_uniform_name(line)));
            } else if line.contains("vec4<f32>") {
                hlsl.push_str(&format!("float4 {};\n", extract_uniform_name(line)));
            } else if line.contains("u32") {
                hlsl.push_str(&format!("uint {};\n", extract_uniform_name(line)));
            }
        }
    }

    hlsl.push_str("\nstruct PSInput {\n");
    hlsl.push_str("    float4 position : SV_POSITION;\n");
    hlsl.push_str("    float2 uv : TEXCOORD;\n");
    hlsl.push_str("};\n\n");

    hlsl.push_str("float4 main(PSInput input) : SV_TARGET {\n");
    hlsl.push_str("    float2 uv = input.uv;\n");

    // Convert WGSL code to HLSL
    let wgsl_body = extract_wgsl_body(wgsl)?;
    let hlsl_body = wgsl_body
        .replace("vec2<f32>", "float2")
        .replace("vec3<f32>", "float3")
        .replace("vec4<f32>", "float4")
        .replace("f32(", "float(")
        .replace("i32(", "int(")
        .replace("u32(", "uint(")
        .replace("bool(", "bool(")
        .replace("textureSample", "tex2D")
        .replace("let ", "")
        .replace(";", " = ")
        .replace("textureStore(output_texture, coords, color)", "return color")
        .replace("gl_FragCoord", "input.position")
        .replace("position.xy", "input.uv");

    hlsl.push_str(&hlsl_body);
    hlsl.push_str("}\n");

    Ok(hlsl)
}

/// Extract uniform variable name from WGSL uniform declaration
fn extract_uniform_name(line: &str) -> &str {
    if let Some(start) = line.find("var<uniform>") {
        let after_var = &line[start + 13..]; // Skip "var<uniform> "
        if let Some(end) = after_var.find(":") {
            return after_var[..end].trim();
        }
    }
    ""
}

/// Extract the main function body from WGSL code
fn extract_wgsl_body(wgsl: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(start) = wgsl.find("fn main(") {
        if let Some(body_start) = wgsl[start..].find('{') {
            if let Some(body_end) = wgsl[start + body_start..].find('}') {
                let body = &wgsl[start + body_start + 1..start + body_start + body_end];
                return Ok(body.to_string());
            }
        }
    }
    Err("Could not extract WGSL main function body".into())
}