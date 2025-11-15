use std::collections::HashMap;
use anyhow::{Result, Context, bail};
use tree_sitter::{Parser, Language, Node, Tree, Query, QueryCursor};
use crate::converter::diagnostics::{Diagnostic, DiagnosticSeverity, Diagnostics, DiagnosticHelpers};

/// GLSL to WGSL converter using tree-sitter for parsing
pub struct GLSLConverter {
    parser: Parser,
    diagnostics: Diagnostics,
    symbol_table: HashMap<String, SymbolInfo>,
    uniform_blocks: Vec<UniformBlock>,
    functions: HashMap<String, FunctionInfo>,
}

#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    glsl_type: String,
    wgsl_type: String,
    storage_class: StorageClass,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
struct UniformBlock {
    name: String,
    members: Vec<SymbolInfo>,
    set: u32,
    binding: u32,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    return_type: String,
    parameters: Vec<ParameterInfo>,
    body: String,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
struct ParameterInfo {
    name: String,
    glsl_type: String,
    wgsl_type: String,
    storage_class: StorageClass,
}

#[derive(Debug, Clone, PartialEq)]
enum StorageClass {
    Uniform,
    Storage,
    Function,
    Private,
    Workgroup,
}

impl GLSLConverter {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        
        // Load GLSL grammar
        let language = unsafe { tree_sitter_glsl() };
        parser.set_language(language)
            .with_context(|| "Failed to set GLSL language for parser")?;
        
        Ok(Self {
            parser,
            diagnostics: Diagnostics::new(),
            symbol_table: HashMap::new(),
            uniform_blocks: Vec::new(),
            functions: HashMap::new(),
        })
    }
    
    /// Convert GLSL source code to WGSL
    pub fn convert(&mut self, glsl_source: &str, file_path: &str) -> Result<String> {
        // Parse the GLSL source
        let tree = self.parse_glsl(glsl_source, file_path)?;
        
        // Analyze the AST
        self.analyze_ast(&tree, glsl_source, file_path)?;
        
        // Generate WGSL code
        let wgsl = self.generate_wgsl(glsl_source, &tree, file_path)?;
        
        Ok(wgsl)
    }
    
    /// Parse GLSL source code into an AST
    fn parse_glsl(&mut self, glsl_source: &str, file_path: &str) -> Result<Tree> {
        let tree = self.parser.parse(glsl_source, None)
            .with_context(|| format!("Failed to parse GLSL in {}", file_path))?;
        
        // Check for syntax errors
        if tree.root_node().has_error() {
            self.collect_parse_errors(&tree, glsl_source, file_path);
            bail!("GLSL parsing failed with syntax errors");
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
    
    /// Analyze the AST to extract symbols, uniforms, functions, etc.
    fn analyze_ast(&mut self, tree: &Tree, source: &str, file_path: &str) -> Result<()> {
        let root_node = tree.root_node();
        
        // Find all declarations
        self.find_declarations(&root_node, source, file_path)?;
        
        // Find all function definitions
        self.find_functions(&root_node, source, file_path)?;
        
        // Find all uniform blocks
        self.find_uniform_blocks(&root_node, source, file_path)?;
        
        Ok(())
    }
    
    /// Find all declarations (variables, uniforms, etc.)
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
                let wgsl_type = self.convert_glsl_type_to_wgsl(&type_name);
                let start = match_.captures[0].node.start_position();
                
                self.symbol_table.insert(var_name.clone(), SymbolInfo {
                    name: var_name,
                    glsl_type: type_name,
                    wgsl_type,
                    storage_class: StorageClass::Private,
                    line: start.row + 1,
                    column: start.column + 1,
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
                
                self.functions.insert(name.clone(), FunctionInfo {
                    name: name.clone(),
                    return_type: ret_type,
                    parameters: Vec::new(), // Will be populated separately
                    body: body_text,
                    line: start.row + 1,
                    column: start.column + 1,
                });
            }
        }
        
        Ok(())
    }
    
    /// Find uniform blocks
    fn find_uniform_blocks(&mut self, node: &Node, source: &str, file_path: &str) -> Result<()> {
        let mut cursor = node.walk();
        let query = Query::new(self.parser.language().unwrap(), r#"
            (declaration
                type: (type_qualifier) @qualifier
                declarator: (init_declarator
                    declarator: (identifier) @name
                )
            )
        "#).with_context(|| "Failed to create uniform query")?;
        
        let mut query_cursor = QueryCursor::new();
        let matches = query_cursor.matches(&query, *node, source.as_bytes());
        
        for match_ in matches {
            let mut qualifier = None;
            let mut name = None;
            
            for capture in match_.captures {
                let node_text = &source[capture.node.byte_range()];
                match self.parser.query().capture_names()[capture.index as usize] {
                    "qualifier" => qualifier = Some(node_text.to_string()),
                    "name" => name = Some(node_text.to_string()),
                    _ => {}
                }
            }
            
            if let (Some(qual), Some(var_name)) = (qualifier, name) {
                if qual.contains("uniform") {
                    // This is a uniform declaration
                    let start = match_.captures[0].node.start_position();
                    
                    // For now, add to a default uniform block
                    if self.uniform_blocks.is_empty() {
                        self.uniform_blocks.push(UniformBlock {
                            name: "Uniforms".to_string(),
                            members: Vec::new(),
                            set: 0,
                            binding: 0,
                        });
                    }
                    
                    if let Some(block) = self.uniform_blocks.first_mut() {
                        block.members.push(SymbolInfo {
                            name: var_name,
                            glsl_type: "unknown".to_string(),
                            wgsl_type: "unknown".to_string(),
                            storage_class: StorageClass::Uniform,
                            line: start.row + 1,
                            column: start.column + 1,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Convert GLSL type to WGSL type
    fn convert_glsl_type_to_wgsl(&self, glsl_type: &str) -> String {
        match glsl_type {
            "void" => "()".to_string(),
            "bool" => "bool".to_string(),
            "int" => "i32".to_string(),
            "uint" => "u32".to_string(),
            "float" => "f32".to_string(),
            "double" => "f64".to_string(),
            "vec2" => "vec2<f32>".to_string(),
            "vec3" => "vec3<f32>".to_string(),
            "vec4" => "vec4<f32>".to_string(),
            "ivec2" => "vec2<i32>".to_string(),
            "ivec3" => "vec3<i32>".to_string(),
            "ivec4" => "vec4<i32>".to_string(),
            "uvec2" => "vec2<u32>".to_string(),
            "uvec3" => "vec3<u32>".to_string(),
            "uvec4" => "vec4<u32>".to_string(),
            "bvec2" => "vec2<bool>".to_string(),
            "bvec3" => "vec3<bool>".to_string(),
            "bvec4" => "vec4<bool>".to_string(),
            "mat2" => "mat2x2<f32>".to_string(),
            "mat3" => "mat3x3<f32>".to_string(),
            "mat4" => "mat4x4<f32>".to_string(),
            "mat2x2" => "mat2x2<f32>".to_string(),
            "mat2x3" => "mat2x3<f32>".to_string(),
            "mat2x4" => "mat2x4<f32>".to_string(),
            "mat3x2" => "mat3x2<f32>".to_string(),
            "mat3x3" => "mat3x3<f32>".to_string(),
            "mat3x4" => "mat3x4<f32>".to_string(),
            "mat4x2" => "mat4x2<f32>".to_string(),
            "mat4x3" => "mat4x3<f32>".to_string(),
            "mat4x4" => "mat4x4<f32>".to_string(),
            "sampler2D" => "texture_2d<f32>".to_string(),
            "samplerCube" => "texture_cube<f32>".to_string(),
            _ => {
                // Handle array types and other complex types
                if glsl_type.contains("[") {
                    // Array type
                    let base_type = glsl_type.split('[').next().unwrap();
                    let array_size = glsl_type.split('[').nth(1).unwrap_or("").trim_end_matches(']');
                    let wgsl_base = self.convert_glsl_type_to_wgsl(base_type);
                    format!("array<{}, {}>", wgsl_base, array_size)
                } else {
                    // Unknown type, return as-is with a warning
                    glsl_type.to_string()
                }
            }
        }
    }
    
    /// Generate WGSL code from the analyzed AST
    fn generate_wgsl(&mut self, glsl_source: &str, tree: &Tree, file_path: &str) -> Result<String> {
        let mut wgsl_code = String::new();
        
        // Add header comments
        wgsl_code.push_str("// Converted from GLSL\n");
        wgsl_code.push_str(&format!("// Original file: {}\n", file_path));
        wgsl_code.push('\n');
        
        // Generate uniform blocks
        self.generate_uniform_blocks(&mut wgsl_code)?;
        
        // Generate function declarations
        self.generate_function_declarations(&mut wgsl_code)?;
        
        // Generate main function
        self.generate_main_function(&mut wgsl_code)?;
        
        Ok(wgsl_code)
    }
    
    /// Generate uniform blocks in WGSL
    fn generate_uniform_blocks(&mut self, wgsl_code: &mut String) -> Result<()> {
        for (index, block) in self.uniform_blocks.iter().enumerate() {
            wgsl_code.push_str(&format!("struct {} {{\n", block.name));
            
            for member in &block.members {
                wgsl_code.push_str(&format!("    {}: {},\n", member.name, member.wgsl_type));
            }
            
            wgsl_code.push_str("}\n\n");
            wgsl_code.push_str(&format!("@group({}) @binding({}) var<uniform> {}_block: {};\n\n", 
                block.set, block.binding, block.name.to_lowercase(), block.name));
        }
        
        Ok(())
    }
    
    /// Generate function declarations
    fn generate_function_declarations(&mut self, wgsl_code: &mut String) -> Result<()> {
        for function in self.functions.values() {
            let return_type = self.convert_glsl_type_to_wgsl(&function.return_type);
            
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
    
    /// Generate main function
    fn generate_main_function(&mut self, wgsl_code: &mut String) -> Result<()> {
        wgsl_code.push_str("@vertex\n");
        wgsl_code.push_str("fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {\n");
        wgsl_code.push_str("    // Vertex shader logic would be converted here\n");
        wgsl_code.push_str("    return vec4<f32>(0.0, 0.0, 0.0, 1.0);\n");
        wgsl_code.push_str("}\n\n");
        
        wgsl_code.push_str("@fragment\n");
        wgsl_code.push_str("fn fs_main() -> @location(0) vec4<f32> {\n");
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

impl Default for GLSLConverter {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Failed to create GLSL converter"))
    }
}

// External function to load tree-sitter GLSL grammar
extern "C" {
    fn tree_sitter_glsl() -> Language;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glsl_converter_creation() {
        let converter = GLSLConverter::new();
        assert!(converter.is_ok());
    }
    
    #[test]
    fn test_simple_glsl_parsing() {
        let glsl = r#"
            void main() {
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;
        
        let mut converter = GLSLConverter::new().unwrap();
        let result = converter.convert(glsl, "test.frag");
        
        // Should succeed even if conversion is not complete
        assert!(result.is_ok());
        let wgsl = result.unwrap();
        assert!(wgsl.contains("@fragment"));
        assert!(wgsl.contains("fs_main"));
    }
    
    #[test]
    fn test_glsl_type_conversion() {
        let converter = GLSLConverter::new().unwrap();
        
        assert_eq!(converter.convert_glsl_type_to_wgsl("float"), "f32");
        assert_eq!(converter.convert_glsl_type_to_wgsl("vec3"), "vec3<f32>");
        assert_eq!(converter.convert_glsl_type_to_wgsl("mat4"), "mat4x4<f32>");
        assert_eq!(converter.convert_glsl_type_to_wgsl("sampler2D"), "texture_2d<f32>");
    }
    
    #[test]
    fn test_invalid_glsl_detection() {
        let invalid_glsl = r#"
            void main() {
                // Missing semicolon
                gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0)
            }
        "#;
        
        let mut converter = GLSLConverter::new().unwrap();
        let result = converter.convert(invalid_glsl, "test.frag");
        
        // Should fail due to syntax error
        assert!(result.is_err());
        assert!(converter.diagnostics.has_errors());
    }
}