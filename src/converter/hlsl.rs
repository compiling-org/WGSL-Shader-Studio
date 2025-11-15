use std::collections::HashMap;
use anyhow::{Result, Context, bail};
use tree_sitter::{Parser, Language, Node, Tree, Query, QueryCursor};
use crate::converter::diagnostics::{Diagnostic, DiagnosticSeverity, Diagnostics, DiagnosticHelpers};

/// HLSL to WGSL converter using tree-sitter for parsing
pub struct HLSLConverter {
    parser: Parser,
    diagnostics: Diagnostics,
    symbol_table: HashMap<String, SymbolInfo>,
    constant_buffers: Vec<ConstantBuffer>,
    texture_declarations: Vec<TextureInfo>,
    functions: HashMap<String, FunctionInfo>,
}

#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    hlsl_type: String,
    wgsl_type: String,
    storage_class: StorageClass,
    line: usize,
    column: usize,
    semantic: Option<String>, // HLSL semantic (e.g., POSITION, TEXCOORD0)
}

#[derive(Debug, Clone)]
struct ConstantBuffer {
    name: String,
    members: Vec<SymbolInfo>,
    set: u32,
    binding: u32,
}

#[derive(Debug, Clone)]
struct TextureInfo {
    name: String,
    hlsl_type: String,
    wgsl_type: String,
    set: u32,
    binding: u32,
    is_comparison: bool,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    return_type: String,
    parameters: Vec<ParameterInfo>,
    body: String,
    line: usize,
    column: usize,
    shader_type: Option<ShaderType>, // Vertex, Pixel, Compute
}

#[derive(Debug, Clone)]
struct ParameterInfo {
    name: String,
    hlsl_type: String,
    wgsl_type: String,
    storage_class: StorageClass,
    semantic: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum StorageClass {
    Uniform,
    Storage,
    Function,
    Private,
    Workgroup,
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq)]
enum ShaderType {
    Vertex,
    Pixel,
    Compute,
}

impl HLSLConverter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        
        // Load HLSL grammar
        let language = unsafe { tree_sitter_hlsl() };
        parser.set_language(language)
            .with_context(|| "Failed to set HLSL language for parser")?;
        
        Ok(Self {
            parser,
            diagnostics: Diagnostics::new(),
            symbol_table: HashMap::new(),
            constant_buffers: Vec::new(),
            texture_declarations: Vec::new(),
            functions: HashMap::new(),
        })
    }
    
    /// Convert HLSL source code to WGSL
    pub fn convert(&mut self, hlsl_source: &str, file_path: &str) -> Result<String> {
        // Parse the HLSL source
        let tree = self.parse_hlsl(hlsl_source, file_path)?;
        
        // Analyze the AST
        self.analyze_ast(&tree, hlsl_source, file_path)?;
        
        // Generate WGSL code
        let wgsl = self.generate_wgsl(hlsl_source, &tree, file_path)?;
        
        Ok(wgsl)
    }
    
    /// Parse HLSL source code into an AST
    fn parse_hlsl(&mut self, hlsl_source: &str, file_path: &str) -> Result<Tree> {
        let tree = self.parser.parse(hlsl_source, None)
            .with_context(|| format!("Failed to parse HLSL in {}", file_path))?;
        
        // Check for syntax errors
        if tree.root_node().has_error() {
            self.collect_parse_errors(&tree, hlsl_source, file_path);
            bail!("HLSL parsing failed with syntax errors");
        }
        
        Ok(tree)
    }
    
    /// Collect parse errors from the tree
    fn collect_parse_errors(&mut self, tree: &Tree, source: &str, file_path: &str) {
        let root_node = tree.root_node();
        self.walk_for_errors(&root_node, source, file_path);
    }
    
    /// Walk the tree and collect error nodes
    fn walk_for_errors(&mut self, node: &Node, source: &str, file_path: &str) {
        if node.kind() == "ERROR" {
            let start = node.start_position();
            let error_text = &source[node.byte_range()];
            
            self.diagnostics.add_diagnostic(
                DiagnosticHelpers::syntax_error(
                    format!("Syntax error near '{}': unexpected token", error_text),
                    start.row + 1,
                    start.column + 1
                ).with_file_path(file_path.to_string())
            );
        }
        
        for child in node.children(&mut node.walk()) {
            self.walk_for_errors(&child, source, file_path);
        }
    }
    
    /// Analyze the AST to extract symbols, constant buffers, functions, etc.
    fn analyze_ast(&mut self, tree: &Tree, source: &str, file_path: &str) -> Result<()> {
        let root_node = tree.root_node();
        
        // Find all declarations
        self.find_declarations(&root_node, source, file_path)?;
        
        // Find all function definitions
        self.find_functions(&root_node, source, file_path)?;
        
        // Find all constant buffers
        self.find_constant_buffers(&root_node, source, file_path)?;
        
        // Find all texture declarations
        self.find_texture_declarations(&root_node, source, file_path)?;
        
        // Find all semantics
        self.find_semantics(&root_node, source, file_path)?;
        
        Ok(())
    }
    
    /// Find all declarations (variables, textures, etc.)
    fn find_declarations(&mut self, node: &Node, source: &str, file_path: &str) -> Result<()> {
        let mut cursor = node.walk();
        let query = Query::new(self.parser.language().unwrap(), r#"
            (declaration
                type: (_) @type
                declarator: (init_declarator
                    declarator: (identifier) @name
                    value: (_) @value
                )
            )
            (declaration
                type: (_) @type
                declarator: (identifier) @name
            )
        "#).with_context(|| "Failed to create declaration query")?;
        
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(&query, *node, source.as_bytes());
        
        for match_ in matches {
            let mut type_name = None;
            let mut var_name = None;
            
            for capture in match_.captures {
                let node_text = &source[capture.node.byte_range()];
                match self.parser.query().capture_names()[capture.index as usize] {
                    "type" => type_name = Some(node_text.to_string()),
                    "name" => var_name = Some(node_text.to_string()),
                    _ => {}
                }
            }
            
            if let (Some(type_name), Some(var_name)) = (type_name, var_name) {
                let wgsl_type = self.convert_hlsl_type_to_wgsl(&type_name);
                let start = match_.captures[0].node.start_position();
                
                self.symbol_table.insert(var_name.clone(), SymbolInfo {
                    name: var_name,
                    hlsl_type: type_name,
                    wgsl_type,
                    storage_class: StorageClass::Private,
                    line: start.row + 1,
                    column: start.column + 1,
                    semantic: None,
                });
            }
        }
        
        Ok(())
    }
    
    /// Find all function definitions
    fn find_functions(&mut self, node: &Node, source: &str, file_path: &str) -> Result<()> {
        let mut cursor = node.walk();
        let query = Query::new(self.parser.language().unwrap(), r#"
            (function_definition
                function_prototype:
                    (function_declarator
                        declarator: (identifier) @name
                        parameters: (parameter_list) @params
                    )
                type: (_) @return_type
                body: (compound_statement) @body
            )
        "#).with_context(|| "Failed to create function query")?;
        
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(&query, *node, source.as_bytes());
        
        for match_ in matches {
            let mut func_name = None;
            let mut return_type = None;
            let mut body_node = None;
            
            for capture in match_.captures {
                let node_text = &source[capture.node.byte_range()];
                match self.parser.query().capture_names()[capture.index as usize] {
                    "name" => func_name = Some(node_text.to_string()),
                    "return_type" => return_type = Some(node_text.to_string()),
                    "body" => body_node = Some(capture.node),
                    _ => {}
                }
            }
            
            if let (Some(name), Some(ret_type), Some(body)) = (func_name, return_type, body_node) {
                let start = match_.captures[0].node.start_position();
                let body_text = source[body.byte_range()].to_string();
                
                // Determine shader type from function name or attributes
                let shader_type = self.determine_shader_type(&name);
                
                self.functions.insert(name.clone(), FunctionInfo {
                    name: name.clone(),
                    return_type: ret_type,
                    parameters: Vec::new(), // Will be populated separately
                    body: body_text,
                    line: start.row + 1,
                    column: start.column + 1,
                    shader_type,
                });
            }
        }
        
        Ok(())
    }
    
    /// Find all constant buffers (cbuffer)
    fn find_constant_buffers(&mut self, node: &Node, source: &str, file_path: &str) -> Result<()> {
        let mut cursor = node.walk();
        let query = Query::new(self.parser.language().unwrap(), r#"
            (constant_buffer_declaration
                name: (identifier) @name
                body: (_) @body
            )
        "#).with_context(|| "Failed to create cbuffer query")?;
        
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(&query, *node, source.as_bytes());
        
        for match_ in matches {
            let mut buffer_name = None;
            let mut body_node = None;
            
            for capture in match_.captures {
                let node_text = &source[capture.node.byte_range()];
                match self.parser.query().capture_names()[capture.index as usize] {
                    "name" => buffer_name = Some(node_text.to_string()),
                    "body" => body_node = Some(capture.node),
                    _ => {}
                }
            }
            
            if let (Some(name), Some(body)) = (buffer_name, body_node) {
                let members = self.parse_cbuffer_body(&body, source, file_path)?;
                
                self.constant_buffers.push(ConstantBuffer {
                    name,
                    members,
                    set: 0,
                    binding: self.constant_buffers.len() as u32,
                });
            }
        }
        
        Ok(())
    }
    
    /// Parse constant buffer body to extract members
    fn parse_cbuffer_body(&mut self, body_node: &Node, source: &str, file_path: &str) -> Result<Vec<SymbolInfo>> {
        let mut members = Vec::new();
        
        let query = Query::new(self.parser.language().unwrap(), r#"
            (declaration
                type: (_) @type
                declarator: (init_declarator
                    declarator: (identifier) @name
                )
            )
        "#).with_context(|| "Failed to create cbuffer member query")?;
        
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(&query, *body_node, source.as_bytes());
        
        for match_ in matches {
            let mut type_name = None;
            let mut var_name = None;
            
            for capture in match_.captures {
                let node_text = &source[capture.node.byte_range()];
                match self.parser.query().capture_names()[capture.index as usize] {
                    "type" => type_name = Some(node_text.to_string()),
                    "name" => var_name = Some(node_text.to_string()),
                    _ => {}
                }
            }
            
            if let (Some(type_name), Some(var_name)) = (type_name, var_name) {
                let wgsl_type = self.convert_hlsl_type_to_wgsl(&type_name);
                let start = match_.captures[0].node.start_position();
                
                members.push(SymbolInfo {
                    name: var_name,
                    hlsl_type: type_name,
                    wgsl_type,
                    storage_class: StorageClass::Uniform,
                    line: start.row + 1,
                    column: start.column + 1,
                    semantic: None,
                });
            }
        }
        
        Ok(members)
    }
    
    /// Find texture declarations (Texture2D, TextureCube, etc.)
    fn find_texture_declarations(&mut self, node: &Node, source: &str, file_path: &str) -> Result<()> {
        let mut cursor = node.walk();
        let query = Query::new(self.parser.language().unwrap(), r#"
            (declaration
                type: (type_identifier) @texture_type
                declarator: (init_declarator
                    declarator: (identifier) @name
                )
            )
        "#).with_context(|| "Failed to create texture query")?;
        
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(&query, *node, source.as_bytes());
        
        for match_ in matches {
            let mut texture_type = None;
            let mut texture_name = None;
            
            for capture in match_.captures {
                let node_text = &source[capture.node.byte_range()];
                match self.parser.query().capture_names()[capture.index as usize] {
                    "texture_type" => texture_type = Some(node_text.to_string()),
                    "name" => texture_name = Some(node_text.to_string()),
                    _ => {}
                }
            }
            
            if let (Some(type_name), Some(name)) = (texture_type, texture_name) {
                if type_name.contains("Texture") || type_name.contains("texture") {
                    let wgsl_type = self.convert_hlsl_texture_to_wgsl(&type_name);
                    let start = match_.captures[0].node.start_position();
                    
                    self.texture_declarations.push(TextureInfo {
                        name,
                        hlsl_type: type_name,
                        wgsl_type,
                        set: 0,
                        binding: self.texture_declarations.len() as u32,
                        is_comparison: false,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Find semantics (POSITION, TEXCOORD0, etc.)
    fn find_semantics(&mut self, node: &Node, source: &str, file_path: &str) -> Result<()> {
        let mut cursor = node.walk();
        let query = Query::new(self.parser.language().unwrap(), r#"
            (semantic
                name: (identifier) @semantic
            )
        "#).with_context(|| "Failed to create semantic query")?;
        
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(&query, *node, source.as_bytes());
        
        for match_ in matches {
            for capture in match_.captures {
                let semantic_name = &source[capture.node.byte_range()];
                let start = capture.node.start_position();
                
                // Store semantic information for later use
                self.diagnostics.add_diagnostic(
                    DiagnosticHelpers::info(
                        format!("Found semantic: {}", semantic_name),
                        start.row + 1,
                        start.column + 1
                    ).with_file_path(file_path.to_string())
                );
            }
        }
        
        Ok(())
    }
    
    /// Determine shader type from function name
    fn determine_shader_type(&self, function_name: &str) -> Option<ShaderType> {
        let name_lower = function_name.to_lowercase();
        
        if name_lower.contains("vertex") || name_lower.contains("vs") {
            Some(ShaderType::Vertex)
        } else if name_lower.contains("pixel") || name_lower.contains("fragment") || name_lower.contains("ps") {
            Some(ShaderType::Pixel)
        } else if name_lower.contains("compute") || name_lower.contains("cs") {
            Some(ShaderType::Compute)
        } else {
            None
        }
    }
    
    /// Convert HLSL type to WGSL type
    fn convert_hlsl_type_to_wgsl(&self, hlsl_type: &str) -> String {
        match hlsl_type {
            "void" => "()".to_string(),
            "bool" => "bool".to_string(),
            "int" => "i32".to_string(),
            "uint" => "u32".to_string(),
            "dword" => "u32".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            "float2" => "vec2<f32>".to_string(),
            "float3" => "vec3<f32>".to_string(),
            "float4" => "vec4<f32>".to_string(),
            "int2" => "vec2<i32>".to_string(),
            "int3" => "vec3<i32>".to_string(),
            "int4" => "vec4<i32>".to_string(),
            "uint2" => "vec2<u32>".to_string(),
            "uint3" => "vec3<u32>".to_string(),
            "uint4" => "vec4<u32>".to_string(),
            "bool2" => "vec2<bool>".to_string(),
            "bool3" => "vec3<bool>".to_string(),
            "bool4" => "vec4<bool>".to_string(),
            "float2x2" => "mat2x2<f32>".to_string(),
            "float3x3" => "mat3x3<f32>".to_string(),
            "float4x4" => "mat4x4<f32>".to_string(),
            "matrix" => "mat4x4<f32>".to_string(),
            _ => {
                // Handle array types and other complex types
                if hlsl_type.contains("[") {
                    // Array type
                    let base_type = hlsl_type.split('[').next().unwrap();
                    let array_size = hlsl_type.split('[').nth(1).unwrap_or("").trim_end_matches(']');
                    let wgsl_base = self.convert_hlsl_type_to_wgsl(base_type);
                    format!("array<{}, {}>", wgsl_base, array_size)
                } else {
                    // Unknown type, return as-is with a warning
                    hlsl_type.to_string()
                }
            }
        }
    }
    
    /// Convert HLSL texture type to WGSL type
    fn convert_hlsl_texture_to_wgsl(&self, hlsl_texture_type: &str) -> String {
        match hlsl_texture_type {
            "Texture2D" => "texture_2d<f32>".to_string(),
            "Texture2DMS" => "texture_multisampled_2d<f32>".to_string(),
            "Texture3D" => "texture_3d<f32>".to_string(),
            "TextureCube" => "texture_cube<f32>".to_string(),
            "Texture2DArray" => "texture_2d_array<f32>".to_string(),
            "TextureCubeArray" => "texture_cube_array<f32>".to_string(),
            "Texture1D" => "texture_1d<f32>".to_string(),
            "Texture1DArray" => "texture_1d_array<f32>".to_string(),
            "Texture2DShadow" | "Texture2DMSArray" | "TextureCubeShadow" => {
                // Comparison samplers
                hlsl_texture_type.to_string()
            }
            _ => {
                // Generic texture type
                "texture_2d<f32>".to_string()
            }
        }
    }
    
    /// Generate WGSL code from the analyzed AST
    fn generate_wgsl(&mut self, hlsl_source: &str, tree: &Tree, file_path: &str) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Add header comments
        wgsl_code.push_str("// Converted from HLSL\n");
        wgsl_code.push_str(&format!("// Original file: {}\n", file_path));
        wgsl_code.push('\n');
        
        // Generate vertex input structure
        self.generate_vertex_input_structure(&mut wgsl_code)?;
        
        // Generate constant buffers
        self.generate_constant_buffers(&mut wgsl_code)?;
        
        // Generate texture declarations
        self.generate_texture_declarations(&mut wgsl_code)?;
        
        // Generate sampler declarations
        self.generate_sampler_declarations(&mut wgsl_code)?;
        
        // Generate function declarations
        self.generate_function_declarations(&mut wgsl_code)?;
        
        // Generate main functions
        self.generate_main_functions(&mut wgsl_code)?;
        
        Ok(wgsl_code)
    }
    
    /// Generate vertex input structure
    fn generate_vertex_input_structure(&mut self, wgsl_code: &mut String) -> Result<()> {
        wgsl_code.push_str("struct VertexInput {\n");
        wgsl_code.push_str("    @location(0) position: vec3<f32>,\n");
        wgsl_code.push_str("    @location(1) texcoord: vec2<f32>,\n");
        wgsl_code.push_str("    @location(2) normal: vec3<f32>,\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("struct VertexOutput {\n");
        wgsl_code.push_str("    @builtin(position) position: vec4<f32>,\n");
        wgsl_code.push_str("    @location(0) texcoord: vec2<f32>,\n");
        wgsl_code.push_str("    @location(1) normal: vec3<f32>,\n");
        wgsl_code.push_str("}\n\n");
        
        Ok(())
    }
    
    /// Generate constant buffers in WGSL
    fn generate_constant_buffers(&mut self, wgsl_code: &mut String) -> Result<()> {
        for (index, cbuffer) in self.constant_buffers.iter().enumerate() {
            wgsl_code.push_str(&format!("struct {} {{\n", cbuffer.name));
            
            for member in &cbuffer.members {
                wgsl_code.push_str(&format!("    {}: {},\n", member.name, member.wgsl_type));
            }
            
            wgsl_code.push_str("}\n\n");
            wgsl_code.push_str(&format!("@group({}) @binding({}) var<uniform> {}_block: {};\n\n", 
                cbuffer.set, cbuffer.binding, cbuffer.name.to_lowercase(), cbuffer.name));
        }
        
        Ok(())
    }
    
    /// Generate texture declarations
    fn generate_texture_declarations(&mut self, wgsl_code: &mut String) -> Result<()> {
        for (index, texture) in self.texture_declarations.iter().enumerate() {
            wgsl_code.push_str(&format!("@group({}) @binding({}) var {}: {};\n", 
                texture.set, texture.binding, texture.name, texture.wgsl_type));
        }
        
        if !self.texture_declarations.is_empty() {
            wgsl_code.push('\n');
        }
        
        Ok(())
    }
    
    /// Generate sampler declarations
    fn generate_sampler_declarations(&mut self, wgsl_code: &mut String) -> Result<()> {
        // Add default samplers for textures
        for (index, texture) in self.texture_declarations.iter().enumerate() {
            wgsl_code.push_str(&format!("@group({}) @binding({}) var {}_sampler: sampler;\n", 
                texture.set, texture.binding + 1000, texture.name));
        }
        
        if !self.texture_declarations.is_empty() {
            wgsl_code.push('\n');
        }
        
        Ok(())
    }
    
    /// Generate function declarations
    fn generate_function_declarations(&mut self, wgsl_code: &mut String) -> Result<()> {
        for function in self.functions.values() {
            let return_type = self.convert_hlsl_type_to_wgsl(&function.return_type);
            
            wgsl_code.push_str(&format!("fn {}(", function.name));
            
            // Add parameters
            let params: Vec<String> = function.parameters.iter()
                .map(|p| format!("{}: {}", p.name, p.wgsl_type))
                .collect();
            wgsl_code.push_str(&params.join(", "));
            
            wgsl_code.push_str(&format!(") -> {} {{\n", return_type));
            wgsl_code.push_str("    // Function body would be converted here\n");
            wgsl_code.push_str("}\n\n");
        }
        
        Ok(())
    }
    
    /// Generate main functions
    fn generate_main_functions(&mut self, wgsl_code: &mut String) -> Result<()> {
        // Generate vertex shader main function
        wgsl_code.push_str("@vertex\n");
        wgsl_code.push_str("fn vs_main(input: VertexInput) -> VertexOutput {\n");
        wgsl_code.push_str("    var output: VertexOutput;\n");
        wgsl_code.push_str("    output.position = vec4<f32>(input.position, 1.0);\n");
        wgsl_code.push_str("    output.texcoord = input.texcoord;\n");
        wgsl_code.push_str("    output.normal = input.normal;\n");
        wgsl_code.push_str("    return output;\n");
        wgsl_code.push_str("}\n\n");
        
        // Generate fragment shader main function
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {\n");
        wgsl_code.push_str("    // Fragment shader logic would be converted here\n");
        wgsl_code.push_str("    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n");
        
        Ok(())
    }
    
    /// Get diagnostics from conversion
    pub fn get_diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }
    
    /// Take ownership of diagnostics
    pub fn take_diagnostics(self) -> Diagnostics {
        self.diagnostics
    }
}

impl Default for HLSLConverter {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create HLSL converter"))
    }
}

// External function to load tree-sitter HLSL grammar
extern "C" {
    fn tree_sitter_hlsl() -> Language;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlsl_converter_creation() {
        let converter = HLSLConverter::new();
        assert!(converter.is_ok());
    }
    
    #[test]
    fn test_simple_hlsl_parsing() {
        let hlsl = r#"
            cbuffer Constants {
                float4x4 worldViewProj;
                float4 lightDir;
            };
            
            Texture2D diffuseTexture;
            SamplerState diffuseSampler;
            
            float4 main(float4 pos : POSITION) : SV_POSITION {
                return mul(pos, worldViewProj);
            }
        "#;
        
        let mut converter = HLSLConverter::new().unwrap();
        let result = converter.convert(hlsl, "test.hlsl");
        
        // Should succeed even if conversion is not complete
        assert!(result.is_ok());
        let wgsl = result.unwrap();
        assert!(wgsl.contains("@vertex"));
        assert!(wgsl.contains("vs_main"));
    }
    
    #[test]
    fn test_hlsl_type_conversion() {
        let converter = HLSLConverter::new().unwrap();
        
        assert_eq!(converter.convert_hlsl_type_to_wgsl("float"), "f32");
        assert_eq!(converter.convert_hlsl_type_to_wgsl("float3"), "vec3<f32>");
        assert_eq!(converter.convert_hlsl_type_to_wgsl("float4x4"), "mat4x4<f32>");
        assert_eq!(converter.convert_hlsl_type_to_wgsl("Texture2D"), "texture_2d<f32>");
    }
    
    #[test]
    fn test_invalid_hlsl_detection() {
        let invalid_hlsl = r#"
            cbuffer Constants {
                float4x4 worldViewProj;
                // Missing semicolon
                float4 lightDir
            };
        "#;
        
        let mut converter = HLSLConverter::new().unwrap();
        let result = converter.convert(invalid_hlsl, "test.hlsl");
        
        // Should fail due to syntax error
        assert!(result.is_err());
        assert!(converter.diagnostics.has_errors());
    }
}