use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

use crate::wgsl_ast_parser::{AstNode, AstVisitor, VisitResult};
use crate::shader_module_system::{ShaderModule, ModuleId};

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
    Hlsl,
    Msl,
    SpirV,
}

impl fmt::Display for ShaderLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShaderLanguage::Wgsl => write!(f, "WGSL"),
            ShaderLanguage::Glsl => write!(f, "GLSL"),
            ShaderLanguage::Hlsl => write!(f, "HLSL"),
            ShaderLanguage::Msl => write!(f, "MSL"),
            ShaderLanguage::SpirV => write!(f, "SPIR-V"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranspilerOptions {
    pub source_language: ShaderLanguage,
    pub target_language: ShaderLanguage,
    pub preserve_comments: bool,
    pub optimize_code: bool,
    pub validate_semantics: bool,
    pub generate_debug_info: bool,
    pub enable_extensions: Vec<String>,
    pub shader_stage: ShaderStage,
    pub profile: TranspilerProfile,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
    Mesh,
    Task,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TranspilerProfile {
    Core,
    Compatibility,
    ES,
    Desktop,
    Mobile,
    Web,
}

impl Default for TranspilerOptions {
    fn default() -> Self {
        Self {
            source_language: ShaderLanguage::Wgsl,
            target_language: ShaderLanguage::Glsl,
            preserve_comments: true,
            optimize_code: false,
            validate_semantics: true,
            generate_debug_info: false,
            enable_extensions: Vec::new(),
            shader_stage: ShaderStage::Fragment,
            profile: TranspilerProfile::Core,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranspilerResult {
    pub source_code: String,
    pub source_map: Option<String>,
    pub dependencies: Vec<String>,
    pub warnings: Vec<TranspilerWarning>,
    pub info_log: String,
    pub metadata: TranspilerMetadata,
}

#[derive(Debug, Clone)]
pub struct TranspilerMetadata {
    pub input_size: usize,
    pub output_size: usize,
    pub transpile_time_ms: f64,
    pub optimization_passes: Vec<String>,
    pub used_extensions: Vec<String>,
    pub uniform_blocks: Vec<UniformBlockInfo>,
    pub texture_bindings: Vec<TextureBindingInfo>,
}

#[derive(Debug, Clone)]
pub struct UniformBlockInfo {
    pub name: String,
    pub binding: u32,
    pub set: u32,
    pub size: usize,
    pub members: Vec<UniformMemberInfo>,
}

#[derive(Debug, Clone)]
pub struct UniformMemberInfo {
    pub name: String,
    pub type_name: String,
    pub offset: usize,
    pub size: usize,
    pub array_size: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TextureBindingInfo {
    pub name: String,
    pub binding: u32,
    pub set: u32,
    pub texture_type: String,
    pub sample_type: String,
}

#[derive(Debug, Error)]
pub enum TranspilerError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Semantic error: {0}")]
    SemanticError(String),
    
    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
    
    #[error("Language mismatch: {0} -> {1}")]
    LanguageMismatch(ShaderLanguage, ShaderLanguage),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Clone)]
pub struct TranspilerWarning {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub severity: WarningSeverity,
    pub category: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

pub type TranspilerResult<T> = Result<T, TranspilerError>;

pub trait ShaderTranspiler: Send + Sync {
    fn transpile(&self, source: &str, options: &TranspilerOptions) -> TranspilerResult<TranspilerResult>;
    fn get_supported_source_languages(&self) -> Vec<ShaderLanguage>;
    fn get_supported_target_languages(&self) -> Vec<ShaderLanguage>;
    fn validate_source(&self, source: &str, language: ShaderLanguage) -> TranspilerResult<()>;
}

pub struct MultiFormatTranspiler {
    transpilers: HashMap<(ShaderLanguage, ShaderLanguage), Box<dyn ShaderTranspiler>>,
    validator: Arc<ShaderValidator>,
    optimizer: Arc<ShaderOptimizer>,
}

impl MultiFormatTranspiler {
    pub fn new() -> Self {
        let mut transpilers = HashMap::new();
        
        transpilers.insert(
            (ShaderLanguage::Wgsl, ShaderLanguage::Glsl),
            Box::new(WgslToGlslTranspiler::new()) as Box<dyn ShaderTranspiler>
        );
        
        transpilers.insert(
            (ShaderLanguage::Glsl, ShaderLanguage::Wgsl),
            Box::new(GlslToWgslTranspiler::new()) as Box<dyn ShaderTranspiler>
        );
        
        transpilers.insert(
            (ShaderLanguage::Wgsl, ShaderLanguage::Hlsl),
            Box::new(WgslToHlslTranspiler::new()) as Box<dyn ShaderTranspiler>
        );
        
        transpilers.insert(
            (ShaderLanguage::Hlsl, ShaderLanguage::Wgsl),
            Box::new(HlslToWgslTranspiler::new()) as Box<dyn ShaderTranspiler>
        );

        Self {
            transpilers,
            validator: Arc::new(ShaderValidator::new()),
            optimizer: Arc::new(ShaderOptimizer::new()),
        }
    }

    pub fn register_transpiler(
        &mut self,
        source: ShaderLanguage,
        target: ShaderLanguage,
        transpiler: Box<dyn ShaderTranspiler>,
    ) {
        self.transpilers.insert((source, target), transpiler);
    }

    pub fn transpile(&self, source: &str, options: &TranspilerOptions) -> TranspilerResult<TranspilerResult> {
        let start_time = std::time::Instant::now();

        if options.validate_semantics {
            self.validator.validate_source(source, options.source_language)?;
        }

        let transpiler = self.transpilers
            .get(&(options.source_language, options.target_language))
            .ok_or_else(|| TranspilerError::LanguageMismatch(
                options.source_language.clone(),
                options.target_language.clone()
            ))?;

        let mut result = transpiler.transpile(source, options)?;

        if options.optimize_code {
            result = self.optimizer.optimize(result, options)?;
        }

        let transpile_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        result.metadata.transpile_time_ms = transpile_time_ms;

        Ok(result)
    }

    pub fn get_supported_conversions(&self) -> Vec<(ShaderLanguage, ShaderLanguage)> {
        self.transpilers.keys().cloned().collect()
    }
}

pub struct WgslToGlslTranspiler {
    visitor: WgslToGlslVisitor,
}

impl WgslToGlslTranspiler {
    pub fn new() -> Self {
        Self {
            visitor: WgslToGlslVisitor::new(),
        }
    }
}

impl ShaderTranspiler for WgslToGlslTranspiler {
    fn transpile(&self, source: &str, options: &TranspilerOptions) -> TranspilerResult<TranspilerResult> {
        use crate::wgsl_ast_parser::WgslAstParser;
        
        let parser = WgslAstParser::new();
        let ast = parser.parse(source)
            .map_err(|e| TranspilerError::ParseError(e.to_string()))?;

        let mut visitor = self.visitor.clone();
        let result = visitor.visit_ast(&ast)?;

        let metadata = self.extract_metadata(&ast, source, &result)?;

        Ok(TranspilerResult {
            source_code: result,
            source_map: None,
            dependencies: Vec::new(),
            warnings: visitor.get_warnings(),
            info_log: visitor.get_info_log(),
            metadata,
        })
    }

    fn get_supported_source_languages(&self) -> Vec<ShaderLanguage> {
        vec![ShaderLanguage::Wgsl]
    }

    fn get_supported_target_languages(&self) -> Vec<ShaderLanguage> {
        vec![ShaderLanguage::Glsl]
    }

    fn validate_source(&self, source: &str, language: ShaderLanguage) -> TranspilerResult<()> {
        if language != ShaderLanguage::Wgsl {
            return Err(TranspilerError::LanguageMismatch(language, ShaderLanguage::Wgsl));
        }
        Ok(())
    }
}

impl WgslToGlslTranspiler {
    fn extract_metadata(&self, ast: &AstNode, source: &str, result: &str) -> TranspilerResult<TranspilerMetadata> {
        let mut metadata = TranspilerMetadata {
            input_size: source.len(),
            output_size: result.len(),
            transpile_time_ms: 0.0,
            optimization_passes: Vec::new(),
            used_extensions: Vec::new(),
            uniform_blocks: Vec::new(),
            texture_bindings: Vec::new(),
        };

        self.extract_uniform_blocks(ast, &mut metadata)?;
        self.extract_texture_bindings(ast, &mut metadata)?;

        Ok(metadata)
    }

    fn extract_uniform_blocks(&self, ast: &AstNode, metadata: &mut TranspilerMetadata) -> TranspilerResult<()> {
        match ast {
            AstNode::TranslationUnit(items) => {
                for item in items {
                    if let AstNode::StructDecl { name, members, .. } = item {
                        if name.starts_with("Uniforms") {
                            let mut block_info = UniformBlockInfo {
                                name: name.clone(),
                                binding: 0,
                                set: 0,
                                size: 0,
                                members: Vec::new(),
                            };

                            for member in members {
                                if let AstNode::StructMember { name: member_name, type_name, .. } = member {
                                    block_info.members.push(UniformMemberInfo {
                                        name: member_name.clone(),
                                        type_name: type_name.clone(),
                                        offset: 0,
                                        size: 0,
                                        array_size: None,
                                    });
                                }
                            }

                            metadata.uniform_blocks.push(block_info);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn extract_texture_bindings(&self, ast: &AstNode, metadata: &mut TranspilerMetadata) -> TranspilerResult<()> {
        match ast {
            AstNode::TranslationUnit(items) => {
                for item in items {
                    if let AstNode::GlobalVarDecl { name, type_name, .. } = item {
                        if type_name.contains("texture") || type_name.contains("Texture") {
                            metadata.texture_bindings.push(TextureBindingInfo {
                                name: name.clone(),
                                binding: 0,
                                set: 0,
                                texture_type: type_name.clone(),
                                sample_type: "float".to_string(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct WgslToGlslVisitor {
    output: String,
    warnings: Vec<TranspilerWarning>,
    info_log: String,
    indent_level: usize,
}

impl WgslToGlslVisitor {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            warnings: Vec::new(),
            info_log: String::new(),
            indent_level: 0,
        }
    }

    pub fn visit_ast(&mut self, ast: &AstNode) -> TranspilerResult<String> {
        self.output.clear();
        self.warnings.clear();
        self.info_log.clear();

        self.visit_node(ast)?;
        Ok(self.output.clone())
    }

    pub fn get_warnings(&self) -> Vec<TranspilerWarning> {
        self.warnings.clone()
    }

    pub fn get_info_log(&self) -> String {
        self.info_log.clone()
    }

    fn visit_node(&mut self, node: &AstNode) -> TranspilerResult<()> {
        match node {
            AstNode::TranslationUnit(items) => {
                self.write_line("#version 450 core");
                self.write_line("");
                
                for item in items {
                    self.visit_node(item)?;
                    self.write_line("");
                }
            }
            AstNode::FunctionDecl { name, params, return_type, body, .. } => {
                self.write_indent();
                self.write(&format!("{} ", self.map_shader_stage(name)));
                self.write(&format!("{}(", name));
                
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.visit_node(param)?;
                }
                
                self.write(")");
                if let Some(ret_type) = return_type {
                    self.write(&format!(" -> {}", self.map_type(ret_type)));
                }
                self.write_line(" {");
                
                self.indent_level += 1;
                self.visit_node(body)?;
                self.indent_level -= 1;
                
                self.write_line("}");
            }
            AstNode::StructDecl { name, members, .. } => {
                self.write_indent();
                self.write_line(&format!("struct {} {{", name));
                
                self.indent_level += 1;
                for member in members {
                    self.visit_node(member)?;
                }
                self.indent_level -= 1;
                
                self.write_line("}");
            }
            AstNode::StructMember { name, type_name, .. } => {
                self.write_indent();
                self.write_line(&format!("{} {};", self.map_type(type_name), name));
            }
            AstNode::GlobalVarDecl { name, type_name, initializer, .. } => {
                self.write_indent();
                self.write(&format!("{} {}", self.map_type(type_name), name));
                if let Some(init) = initializer {
                    self.write(&format!(" = {}", self.map_expression(init)));
                }
                self.write_line(";");
            }
            AstNode::BlockStatement(statements) => {
                for stmt in statements {
                    self.visit_node(stmt)?;
                }
            }
            AstNode::ReturnStatement(expr) => {
                self.write_indent();
                if let Some(expr) = expr {
                    self.write_line(&format!("return {};", self.map_expression(expr)));
                } else {
                    self.write_line("return;");
                }
            }
            AstNode::AssignmentStatement { target, value } => {
                self.write_indent();
                self.write_line(&format!("{} = {};", self.map_expression(target), self.map_expression(value)));
            }
            AstNode::IfStatement { condition, then_branch, else_branch } => {
                self.write_indent();
                self.write_line(&format!("if ({}) {{", self.map_expression(condition)));
                
                self.indent_level += 1;
                self.visit_node(then_branch)?;
                self.indent_level -= 1;
                
                if let Some(else_branch) = else_branch {
                    self.write_line("} else {");
                    self.indent_level += 1;
                    self.visit_node(else_branch)?;
                    self.indent_level -= 1;
                }
                
                self.write_line("}");
            }
            _ => {
                self.add_warning(1, 1, format!("Unsupported AST node: {:?}", node), "transpiler");
            }
        }
        Ok(())
    }

    fn map_shader_stage(&self, name: &str) -> &str {
        if name.contains("vs_") || name.contains("vertex") {
            "vertex"
        } else if name.contains("fs_") || name.contains("fragment") {
            "fragment"
        } else if name.contains("cs_") || name.contains("compute") {
            "compute"
        } else {
            ""
        }
    }

    fn map_type(&self, wgsl_type: &str) -> String {
        match wgsl_type {
            "f32" => "float".to_string(),
            "i32" => "int".to_string(),
            "u32" => "uint".to_string(),
            "bool" => "bool".to_string(),
            "vec2<f32>" => "vec2".to_string(),
            "vec3<f32>" => "vec3".to_string(),
            "vec4<f32>" => "vec4".to_string(),
            "mat4x4<f32>" => "mat4".to_string(),
            _ => {
                self.add_warning(1, 1, format!("Unknown type mapping for: {}", wgsl_type), "type-mapping");
                wgsl_type.to_string()
            }
        }
    }

    fn map_expression(&self, expr: &str) -> String {
        expr.to_string()
    }

    fn write(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn write_line(&mut self, text: &str) {
        self.write_indent();
        self.output.push_str(text);
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn add_warning(&mut self, line: usize, column: usize, message: String, category: &str) {
        self.warnings.push(TranspilerWarning {
            line,
            column,
            message,
            severity: WarningSeverity::Warning,
            category: category.to_string(),
        });
    }
}

pub struct GlslToWgslTranspiler {
    visitor: GlslToWgslVisitor,
}

impl GlslToWgslTranspiler {
    pub fn new() -> Self {
        Self {
            visitor: GlslToWgslVisitor::new(),
        }
    }
}

impl ShaderTranspiler for GlslToWgslTranspiler {
    fn transpile(&self, source: &str, options: &TranspilerOptions) -> TranspilerResult<TranspilerResult> {
        let mut visitor = self.visitor.clone();
        let result = visitor.visit_glsl(source)?;

        Ok(TranspilerResult {
            source_code: result,
            source_map: None,
            dependencies: Vec::new(),
            warnings: visitor.get_warnings(),
            info_log: visitor.get_info_log(),
            metadata: TranspilerMetadata {
                input_size: source.len(),
                output_size: result.len(),
                transpile_time_ms: 0.0,
                optimization_passes: Vec::new(),
                used_extensions: Vec::new(),
                uniform_blocks: Vec::new(),
                texture_bindings: Vec::new(),
            },
        })
    }

    fn get_supported_source_languages(&self) -> Vec<ShaderLanguage> {
        vec![ShaderLanguage::Glsl]
    }

    fn get_supported_target_languages(&self) -> Vec<ShaderLanguage> {
        vec![ShaderLanguage::Wgsl]
    }

    fn validate_source(&self, source: &str, language: ShaderLanguage) -> TranspilerResult<()> {
        if language != ShaderLanguage::Glsl {
            return Err(TranspilerError::LanguageMismatch(language, ShaderLanguage::Glsl));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct GlslToWgslVisitor {
    output: String,
    warnings: Vec<TranspilerWarning>,
    info_log: String,
}

impl GlslToWgslVisitor {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            warnings: Vec::new(),
            info_log: String::new(),
        }
    }

    pub fn visit_glsl(&mut self, source: &str) -> TranspilerResult<String> {
        self.output.clear();
        self.warnings.clear();
        self.info_log.clear();

        self.output.push_str(&format!("// Transpiled from GLSL\n// Original size: {} bytes\n\n", source.len()));
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            if trimmed.starts_with("#version") {
                continue;
            }
            
            if trimmed.starts_with("#extension") {
                self.add_warning(1, 1, format!("Extension ignored: {}", trimmed), "extension");
                continue;
            }
            
            let converted = self.convert_glsl_line(trimmed);
            self.output.push_str(&converted);
            self.output.push('\n');
        }

        Ok(self.output.clone())
    }

    pub fn get_warnings(&self) -> Vec<TranspilerWarning> {
        self.warnings.clone()
    }

    pub fn get_info_log(&self) -> String {
        self.info_log.clone()
    }

    fn convert_glsl_line(&mut self, line: &str) -> String {
        let mut result = line.to_string();
        
        result = result.replace("attribute ", "@location(0) ");
        result = result.replace("varying ", "@location(0) ");
        result = result.replace("uniform ", "@group(0) @binding(0) ");
        result = result.replace("texture2D", "textureSample");
        result = result.replace("vec2", "vec2<f32>");
        result = result.replace("vec3", "vec3<f32>");
        result = result.replace("vec4", "vec4<f32>");
        result = result.replace("mat4", "mat4x4<f32>");
        result = result.replace("float", "f32");
        result = result.replace("int", "i32");
        result = result.replace("void", "");
        
        if result.contains("main()") && !result.contains("fn") {
            result = result.replace("main()", "fn main()");
        }
        
        if result.contains("gl_Position") {
            result = result.replace("gl_Position", "@builtin(position) var<out> position: vec4<f32>");
            self.add_warning(1, 1, "Manual gl_Position conversion required".to_string(), "builtin");
        }
        
        if result.contains("gl_FragColor") {
            result = result.replace("gl_FragColor", "@location(0) var<out> fragColor: vec4<f32>");
            self.add_warning(1, 1, "Manual gl_FragColor conversion required".to_string(), "builtin");
        }

        result
    }

    fn add_warning(&mut self, line: usize, column: usize, message: String, category: &str) {
        self.warnings.push(TranspilerWarning {
            line,
            column,
            message,
            severity: WarningSeverity::Warning,
            category: category.to_string(),
        });
    }
}

pub struct WgslToHlslTranspiler;

impl WgslToHlslTranspiler {
    pub fn new() -> Self {
        Self
    }
}

impl ShaderTranspiler for WgslToHlslTranspiler {
    fn transpile(&self, source: &str, options: &TranspilerOptions) -> TranspilerResult<TranspilerResult> {
        let mut output = String::new();
        output.push_str("// Transpiled from WGSL to HLSL\n");
        output.push_str("// Original size: ");
        output.push_str(&source.len().to_string());
        output.push_str(" bytes\n\n");
        
        for line in source.lines() {
            let converted = self.convert_wgsl_to_hlsl_line(line);
            output.push_str(&converted);
            output.push('\n');
        }

        Ok(TranspilerResult {
            source_code: output,
            source_map: None,
            dependencies: Vec::new(),
            warnings: vec![TranspilerWarning {
                line: 1,
                column: 1,
                message: "Basic WGSL to HLSL conversion - manual review required".to_string(),
                severity: WarningSeverity::Info,
                category: "transpiler".to_string(),
            }],
            info_log: "WGSL to HLSL transpilation completed".to_string(),
            metadata: TranspilerMetadata {
                input_size: source.len(),
                output_size: output.len(),
                transpile_time_ms: 0.0,
                optimization_passes: Vec::new(),
                used_extensions: Vec::new(),
                uniform_blocks: Vec::new(),
                texture_bindings: Vec::new(),
            },
        })
    }

    fn get_supported_source_languages(&self) -> Vec<ShaderLanguage> {
        vec![ShaderLanguage::Wgsl]
    }

    fn get_supported_target_languages(&self) -> Vec<ShaderLanguage> {
        vec![ShaderLanguage::Hlsl]
    }

    fn validate_source(&self, source: &str, language: ShaderLanguage) -> TranspilerResult<()> {
        if language != ShaderLanguage::Wgsl {
            return Err(TranspilerError::LanguageMismatch(language, ShaderLanguage::Wgsl));
        }
        Ok(())
    }
}

impl WgslToHlslTranspiler {
    fn convert_wgsl_to_hlsl_line(&self, line: &str) -> String {
        let mut result = line.to_string();
        
        result = result.replace("vec2<f32>", "float2");
        result = result.replace("vec3<f32>", "float3");
        result = result.replace("vec4<f32>", "float4");
        result = result.replace("mat4x4<f32>", "float4x4");
        result = result.replace("f32", "float");
        result = result.replace("i32", "int");
        result = result.replace("u32", "uint");
        result = result.replace("texture_2d", "Texture2D");
        result = result.replace("sampler", "SamplerState");
        result = result.replace("@group(0) @binding(0)", "");
        result = result.replace("@location(0)", "");
        result = result.replace("@builtin(position)", "SV_Position");
        result = result.replace("@builtin(frag_coord)", "SV_Position");
        
        result
    }
}

pub struct HlslToWgslTranspiler;

impl HlslToWgslTranspiler {
    pub fn new() -> Self {
        Self
    }
}

impl ShaderTranspiler for HlslToWgslTranspiler {
    fn transpile(&self, source: &str, options: &TranspilerOptions) -> TranspilerResult<TranspilerResult> {
        let mut output = String::new();
        output.push_str("// Transpiled from HLSL to WGSL\n");
        output.push_str("// Original size: ");
        output.push_str(&source.len().to_string());
        output.push_str(" bytes\n\n");
        
        for line in source.lines() {
            let converted = self.convert_hlsl_to_wgsl_line(line);
            output.push_str(&converted);
            output.push('\n');
        }

        Ok(TranspilerResult {
            source_code: output,
            source_map: None,
            dependencies: Vec::new(),
            warnings: vec![TranspilerWarning {
                line: 1,
                column: 1,
                message: "Basic HLSL to WGSL conversion - manual review required".to_string(),
                severity: WarningSeverity::Info,
                category: "transpiler".to_string(),
            }],
            info_log: "HLSL to WGSL transpilation completed".to_string(),
            metadata: TranspilerMetadata {
                input_size: source.len(),
                output_size: output.len(),
                transpile_time_ms: 0.0,
                optimization_passes: Vec::new(),
                used_extensions: Vec::new(),
                uniform_blocks: Vec::new(),
                texture_bindings: Vec::new(),
            },
        })
    }

    fn get_supported_source_languages(&self) -> Vec<ShaderLanguage> {
        vec![ShaderLanguage::Hlsl]
    }

    fn get_supported_target_languages(&self) -> Vec<ShaderLanguage> {
        vec![ShaderLanguage::Wgsl]
    }

    fn validate_source(&self, source: &str, language: ShaderLanguage) -> TranspilerResult<()> {
        if language != ShaderLanguage::Hlsl {
            return Err(TranspilerError::LanguageMismatch(language, ShaderLanguage::Hlsl));
        }
        Ok(())
    }
}

impl HlslToWgslTranspiler {
    fn convert_hlsl_to_wgsl_line(&self, line: &str) -> String {
        let mut result = line.to_string();
        
        result = result.replace("float2", "vec2<f32>");
        result = result.replace("float3", "vec3<f32>");
        result = result.replace("float4", "vec4<f32>");
        result = result.replace("float4x4", "mat4x4<f32>");
        result = result.replace("float", "f32");
        result = result.replace("int", "i32");
        result = result.replace("uint", "u32");
        result = result.replace("Texture2D", "texture_2d");
        result = result.replace("SamplerState", "sampler");
        result = result.replace("SV_Position", "@builtin(position)");
        
        result
    }
}

pub struct ShaderValidator {
    validation_rules: Vec<Box<dyn Fn(&str, ShaderLanguage) -> TranspilerResult<()>>>,
}

impl ShaderValidator {
    pub fn new() -> Self {
        Self {
            validation_rules: vec![
                Box::new(|source, language| Self::validate_syntax(source, language)),
                Box::new(|source, language| Self::validate_semantics(source, language)),
            ],
        }
    }

    pub fn validate_source(&self, source: &str, language: ShaderLanguage) -> TranspilerResult<()> {
        for rule in &self.validation_rules {
            rule(source, language)?;
        }
        Ok(())
    }

    fn validate_syntax(source: &str, language: ShaderLanguage) -> TranspilerResult<()> {
        if source.is_empty() {
            return Err(TranspilerError::ValidationError("Empty source code".to_string()));
        }
        Ok(())
    }

    fn validate_semantics(source: &str, language: ShaderLanguage) -> TranspilerResult<()> {
        if source.len() > 1_000_000 {
            return Err(TranspilerError::ValidationError("Source code too large".to_string()));
        }
        Ok(())
    }
}

pub struct ShaderOptimizer {
    optimization_passes: Vec<Box<dyn Fn(String) -> TranspilerResult<String>>>,
}

impl ShaderOptimizer {
    pub fn new() -> Self {
        Self {
            optimization_passes: vec![
                Box::new(|code| Self::remove_unused_variables(code)),
                Box::new(|code| Self::simplify_expressions(code)),
            ],
        }
    }

    pub fn optimize(&self, mut result: TranspilerResult, options: &TranspilerOptions) -> TranspilerResult<TranspilerResult> {
        let mut optimization_passes = Vec::new();
        
        for pass in &self.optimization_passes {
            result.source_code = pass(result.source_code)?;
            optimization_passes.push("applied".to_string());
        }

        result.metadata.optimization_passes = optimization_passes;
        Ok(result)
    }

    fn remove_unused_variables(code: String) -> TranspilerResult<String> {
        Ok(code)
    }

    fn simplify_expressions(code: String) -> TranspilerResult<String> {
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wgsl_to_glsl_transpiler() {
        let transpiler = WgslToGlslTranspiler::new();
        let wgsl_source = r#"
            struct VertexInput {
                @location(0) position: vec3<f32>,
                @location(1) uv: vec2<f32>,
            }

            @vertex
            fn vs_main(input: VertexInput) -> @builtin(position) vec4<f32> {
                return vec4<f32>(input.position, 1.0);
            }
        "#;

        let options = TranspilerOptions {
            source_language: ShaderLanguage::Wgsl,
            target_language: ShaderLanguage::Glsl,
            ..Default::default()
        };

        let result = transpiler.transpile(wgsl_source, &options);
        assert!(result.is_ok());
        
        let transpiled = result.unwrap();
        assert!(transpiled.source_code.contains("#version 450 core"));
        assert!(transpiled.source_code.contains("struct VertexInput"));
    }

    #[test]
    fn test_glsl_to_wgsl_transpiler() {
        let transpiler = GlslToWgslTranspiler::new();
        let glsl_source = r#"
            #version 450 core
            
            attribute vec3 position;
            attribute vec2 uv;
            
            uniform mat4 mvpMatrix;
            
            void main() {
                gl_Position = mvpMatrix * vec4(position, 1.0);
            }
        "#;

        let options = TranspilerOptions {
            source_language: ShaderLanguage::Glsl,
            target_language: ShaderLanguage::Wgsl,
            ..Default::default()
        };

        let result = transpiler.transpile(glsl_source, &options);
        assert!(result.is_ok());
        
        let transpiled = result.unwrap();
        assert!(transpiled.source_code.contains("vec3<f32>"));
    }

    #[test]
    fn test_multi_format_transpiler() {
        let mut transpiler = MultiFormatTranspiler::new();
        let wgsl_source = "fn test() -> f32 { return 1.0; }";

        let options = TranspilerOptions {
            source_language: ShaderLanguage::Wgsl,
            target_language: ShaderLanguage::Glsl,
            ..Default::default()
        };

        let result = transpiler.transpile(wgsl_source, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_transpiler_validation() {
        let validator = ShaderValidator::new();
        
        let result = validator.validate_source("valid shader code", ShaderLanguage::Wgsl);
        assert!(result.is_ok());
        
        let result = validator.validate_source("", ShaderLanguage::Wgsl);
        assert!(result.is_err());
    }
}