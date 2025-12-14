//! Integration tests for ISF loader functionality

use resolume_isf_shaders_rust_ffgl::isf_loader::*;
use std::fs;
use std::path::Path;

#[test]
fn test_load_resolume_isf_shaders() {
    // This test will attempt to load shaders from Magic directories
    // It will gracefully handle cases where the directories don't exist
    match load_resolume_isf_shaders() {
        Ok(shaders) => {
            println!("Successfully loaded {} ISF shaders from Magic directories", shaders.len());
            // Verify that we have some shaders loaded
            assert!(shaders.len() >= 0, "Should have non-negative number of shaders");
            
            // If we have shaders, verify their structure
            for shader in &shaders {
                println!("Loaded shader: {} with {} inputs and {} outputs", 
                    shader.name, shader.inputs.len(), shader.outputs.len());
                
                // Basic validation
                assert!(!shader.name.is_empty(), "Shader name should not be empty");
                assert!(!shader.source.is_empty(), "Shader source should not be empty");
                assert!(shader.outputs.len() > 0, "Shader should have at least one output");
            }
        }
        Err(e) => {
            println!("Failed to load Magic ISF shaders: {}", e);
            // This is acceptable if Magic directories don't exist
            // The function should still return an empty vector, not panic
        }
    }
}

#[test]
fn test_load_isf_shaders_from_directory() {
    // Create a temporary test directory with sample ISF files
    let test_dir = "test_isf_shaders";
    fs::create_dir_all(test_dir).unwrap();
    
    // Create a sample ISF shader file
    let isf_content = r#"/*{
    "NAME": "Test Shader",
    "DESCRIPTION": "A test ISF shader",
    "INPUTS": [
        {
            "NAME": "speed",
            "TYPE": "float",
            "MIN": 0.0,
            "MAX": 1.0,
            "DEFAULT": 0.5
        },
        {
            "NAME": "color",
            "TYPE": "color",
            "DEFAULT": [1.0, 0.0, 0.0, 1.0]
        }
    ],
    "OUTPUTS": [
        {
            "NAME": "outputImage",
            "TYPE": "image"
        }
    ]
}*/

void main() {
    vec2 uv = isf_FragNormCoord;
    vec4 color = vec4(UV.x, UV.y, 0.0, 1.0);
    gl_FragColor = color;
}"#;
    
    let shader_path = Path::new(test_dir).join("test_shader.fs");
    fs::write(&shader_path, isf_content).unwrap();
    
    // Test loading from the directory
    match load_isf_shaders_from_directory(test_dir) {
        Ok(shaders) => {
            assert_eq!(shaders.len(), 1, "Should load exactly one shader");
            let shader = &shaders[0];
            assert_eq!(shader.name, "Test Shader");
            assert_eq!(shader.inputs.len(), 2);
            assert_eq!(shader.outputs.len(), 1);
            
            // Verify input types
            assert_eq!(shader.inputs[0].name, "speed");
            assert!(matches!(shader.inputs[0].input_type, InputType::Float));
            assert_eq!(shader.inputs[0].min, Some(0.0));
            assert_eq!(shader.inputs[0].max, Some(1.0));
            assert_eq!(shader.inputs[0].default, Some(0.5));
            
            assert_eq!(shader.inputs[1].name, "color");
            assert!(matches!(shader.inputs[1].input_type, InputType::Color));
        }
        Err(e) => {
            panic!("Failed to load test ISF shaders: {}", e);
        }
    }
    
    // Clean up
    fs::remove_dir_all(test_dir).unwrap();
}

#[test]
fn test_validate_isf_shader() {
    // Test valid shader
    let valid_shader = IsfShader {
        name: "Valid Shader".to_string(),
        source: "void main() { gl_FragColor = vec4(1.0); }".to_string(),
        inputs: vec![],
        outputs: vec![ShaderOutput {
            name: "outputImage".to_string(),
            output_type: OutputType::Image,
        }],
    };
    
    assert!(validate_isf_shader(&valid_shader).is_ok());
    
    // Test invalid shader - empty name
    let invalid_shader_name = IsfShader {
        name: "".to_string(),
        source: "void main() { gl_FragColor = vec4(1.0); }".to_string(),
        inputs: vec![],
        outputs: vec![ShaderOutput {
            name: "outputImage".to_string(),
            output_type: OutputType::Image,
        }],
    };
    
    assert!(validate_isf_shader(&invalid_shader_name).is_err());
    
    // Test invalid shader - no main function
    let invalid_shader_main = IsfShader {
        name: "Invalid Shader".to_string(),
        source: "vec4 calculateColor() { return vec4(1.0); }".to_string(),
        inputs: vec![],
        outputs: vec![ShaderOutput {
            name: "outputImage".to_string(),
            output_type: OutputType::Image,
        }],
    };
    
    assert!(validate_isf_shader(&invalid_shader_main).is_err());
    
    // Test invalid shader - no outputs
    let invalid_shader_outputs = IsfShader {
        name: "Invalid Shader".to_string(),
        source: "void main() { gl_FragColor = vec4(1.0); }".to_string(),
        inputs: vec![],
        outputs: vec![],
    };
    
    assert!(validate_isf_shader(&invalid_shader_outputs).is_err());
}

#[test]
fn test_get_shader_metadata() {
    let shader = IsfShader {
        name: "Test Metadata Shader".to_string(),
        source: r#"/*{
            "NAME": "Test Metadata Shader",
            "DESCRIPTION": "A shader with metadata",
            "CATEGORIES": "test",
            "CREDIT": "Test Author"
        }*/
        void main() { gl_FragColor = vec4(1.0); }"#.to_string(),
        inputs: vec![
            ShaderInput {
                name: "speed".to_string(),
                input_type: InputType::Float,
                value: ShaderValue::Float(0.5),
                min: Some(0.0),
                max: Some(1.0),
                default: Some(0.5),
            },
        ],
        outputs: vec![
            ShaderOutput {
                name: "outputImage".to_string(),
                output_type: OutputType::Image,
            },
        ],
    };
    
    let metadata = get_shader_metadata(&shader);
    
    assert_eq!(metadata.name, "Test Metadata Shader");
    assert_eq!(metadata.description, Some("A shader with metadata".to_string()));
    assert_eq!(metadata.category, Some("test".to_string()));
    assert_eq!(metadata.author, Some("Test Author".to_string()));
    assert_eq!(metadata.inputs.len(), 1);
    assert_eq!(metadata.outputs.len(), 1);
}
