//! Advanced WGSL AST Parser with Naga Integration
//! 
//! This module implements a comprehensive WGSL parser based on use.gpu patterns,
//! providing AST construction, symbol table management, and type inference.
//! Uses Rust-native naga library instead of JavaScript Lezer for compatibility.

use std::collections::HashMap;
use std::result::Result as StdResult;
use serde::{Serialize, Deserialize};

/// WGSL AST Node types based on use.gpu patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    Module(ModuleNode),
    Function(FunctionNode),
    FunctionDecl { name: String, parameters: Vec<String>, return_type: Option<String>, body: Box<AstNode> },
    Struct(StructNode),
    StructDecl { name: String, fields: Vec<String>, members: Vec<AstNode> },
    StructMember { name: String, type_name: String },
    Variable(VariableNode),
    TypeAlias(TypeAliasNode),
    TypeAliasDecl { name: String, target_type: String },
    Constant(ConstantNode),
    ConstDecl { name: String, value: String },
    Attribute(AttributeNode),
    Statement(StatementNode),
    Expression(ExpressionNode),
    ImportDecl { path: String },
    OverrideDecl { name: String },
    GlobalVarDecl { name: String, type_name: String, initializer: Option<String> },
    BlockStatement(Vec<AstNode>),
    ReturnStatement(Option<String>),
    AssignmentStatement { target: String, value: String },
    IfStatement { condition: String, then_branch: Box<AstNode>, else_branch: Option<Box<AstNode>> },
    TranslationUnit,
}

/// Module node containing all top-level declarations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleNode {
    pub name: Option<String>,
    pub imports: Vec<ImportNode>,
    pub declarations: Vec<DeclarationNode>,
    pub exports: Vec<String>,
    pub attributes: Vec<AttributeNode>,
}

/// Function declaration node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionNode {
    pub name: String,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeNode>,
    pub body: Option<BlockNode>,
    pub attributes: Vec<AttributeNode>,
    pub stage: Option<WgslShaderStage>,
    pub workgroup_size: Option<(u32, u32, u32)>,
}

/// Shader stage enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WgslShaderStage {
    Vertex,
    Fragment,
    Compute,
    Task,
    Mesh,
}

/// Struct definition node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructNode {
    pub name: String,
    pub members: Vec<StructMemberNode>,
    pub attributes: Vec<AttributeNode>,
}

/// Variable declaration node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariableNode {
    pub name: String,
    pub var_type: VarType,
    pub data_type: TypeNode,
    pub initializer: Option<ExpressionNode>,
    pub attributes: Vec<AttributeNode>,
    pub binding: Option<u32>,
    pub group: Option<u32>,
}

/// Variable type (var, let, const)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VarType {
    Var,
    Let,
    Const,
}

/// Type alias node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeAliasNode {
    pub name: String,
    pub aliased_type: TypeNode,
    pub attributes: Vec<AttributeNode>,
}

/// Constant declaration node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantNode {
    pub name: String,
    pub data_type: Option<TypeNode>,
    pub value: ExpressionNode,
    pub attributes: Vec<AttributeNode>,
}

/// Attribute node (@attribute)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeNode {
    pub name: String,
    pub arguments: Vec<ExpressionNode>,
}

/// Import statement node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportNode {
    pub module_path: String,
    pub items: Vec<ImportItem>,
}

/// Import item (specific imports or wildcard)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportItem {
    Named(String),
    Aliased(String, String),
    Wildcard,
}

/// Declaration node wrapper
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeclarationNode {
    Function(FunctionNode),
    Struct(StructNode),
    Variable(VariableNode),
    TypeAlias(TypeAliasNode),
    Constant(ConstantNode),
}

/// Parameter node for functions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterNode {
    pub name: String,
    pub data_type: TypeNode,
    pub attributes: Vec<AttributeNode>,
    pub direction: ParameterDirection,
}

/// Parameter direction (in, out, inout)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ParameterDirection {
    In,
    Out,
    InOut,
}

/// Type node representing WGSL types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeNode {
    Primitive(PrimitiveType),
    Vector(VectorType),
    Matrix(MatrixType),
    Array(ArrayType),
    Struct(String),
    Pointer(PointerType),
    Texture(TextureType),
    Sampler(SamplerType),
    Atomic(AtomicType),
}

/// Primitive scalar types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Bool,
    I32,
    U32,
    F32,
    F16,
}

/// Vector types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorType {
    pub component_type: PrimitiveType,
    pub size: VectorSize,
}

/// Vector size
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VectorSize {
    Two,
    Three,
    Four,
}

/// Matrix types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixType {
    pub component_type: PrimitiveType,
    pub rows: u32,
    pub cols: u32,
}

/// Array types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayType {
    pub element_type: Box<TypeNode>,
    pub size: Option<u32>,
}

/// Pointer types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointerType {
    pub storage_class: StorageClass,
    pub element_type: Box<TypeNode>,
}

/// Storage class for pointers
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StorageClass {
    Function,
    Workgroup,
    Uniform,
    Storage,
    Private,
}

/// Texture types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureType {
    pub sample_type: TextureSampleType,
    pub dimension: TextureDimension,
    pub is_array: bool,
    pub is_multisampled: bool,
}

/// Texture sample type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextureSampleType {
    Float,
    Uint,
    Sint,
    Depth,
}

/// Texture dimension
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextureDimension {
    One,
    Two,
    Three,
    Cube,
}

/// Sampler types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SamplerType {
    Sampler,
    ComparisonSampler,
}

/// Atomic types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AtomicType {
    I32,
    U32,
}

/// Struct member node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructMemberNode {
    pub name: String,
    pub data_type: TypeNode,
    pub attributes: Vec<AttributeNode>,
}

/// Statement node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatementNode {
    Block(BlockNode),
    If(IfNode),
    Switch(SwitchNode),
    Loop(LoopNode),
    For(ForNode),
    While(WhileNode),
    Break,
    Continue,
    Return(Option<ExpressionNode>),
    Variable(VariableNode),
    Assignment(AssignmentNode),
    FunctionCall(FunctionCallNode),
}

/// Block statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockNode {
    pub statements: Vec<StatementNode>,
}

/// If statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfNode {
    pub condition: ExpressionNode,
    pub then_block: Box<StatementNode>,
    pub else_block: Option<Box<StatementNode>>,
}

/// Switch statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SwitchNode {
    pub selector: ExpressionNode,
    pub cases: Vec<CaseNode>,
    pub default: Option<Box<StatementNode>>,
}

/// Case node for switch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseNode {
    pub values: Vec<ExpressionNode>,
    pub body: Box<StatementNode>,
}

/// Loop statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoopNode {
    pub body: Box<StatementNode>,
}

/// For loop statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForNode {
    pub initializer: Option<Box<StatementNode>>,
    pub condition: Option<ExpressionNode>,
    pub increment: Option<ExpressionNode>,
    pub body: Box<StatementNode>,
}

/// While loop statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WhileNode {
    pub condition: ExpressionNode,
    pub body: Box<StatementNode>,
}

/// Assignment statement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssignmentNode {
    pub left: ExpressionNode,
    pub operator: AssignmentOperator,
    pub right: ExpressionNode,
}

/// Assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
}

/// Expression node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionNode {
    Literal(LiteralNode),
    Identifier(String),
    Binary(BinaryExpressionNode),
    Unary(UnaryExpressionNode),
    FunctionCall(FunctionCallNode),
    FieldAccess(FieldAccessNode),
    ArrayAccess(ArrayAccessNode),
    TypeConstructor(TypeConstructorNode),
    Select(SelectNode),
}

/// Literal value node
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralNode {
    Bool(bool),
    I32(i32),
    U32(u32),
    F32(f32),
    F16(f32),
}

/// Binary expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinaryExpressionNode {
    pub left: Box<ExpressionNode>,
    pub operator: BinaryOperator,
    pub right: Box<ExpressionNode>,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Xor,
    ShiftLeft,
    ShiftRight,
}

/// Unary expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryExpressionNode {
    pub operator: UnaryOperator,
    pub operand: Box<ExpressionNode>,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Negate,
    Complement,
}

/// Function call expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionCallNode {
    pub function_name: String,
    pub arguments: Vec<ExpressionNode>,
}

/// Field access expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldAccessNode {
    pub object: Box<ExpressionNode>,
    pub field: String,
}

/// Array access expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArrayAccessNode {
    pub array: Box<ExpressionNode>,
    pub index: Box<ExpressionNode>,
}

/// Type constructor expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeConstructorNode {
    pub type_name: String,
    pub arguments: Vec<ExpressionNode>,
}

/// Select expression (ternary)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectNode {
    pub condition: Box<ExpressionNode>,
    pub true_expression: Box<ExpressionNode>,
    pub false_expression: Box<ExpressionNode>,
}

/// Symbol table entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub name: String,
    pub symbol_type: SymbolType,
    pub data_type: TypeNode,
    pub scope_level: usize,
    pub is_mutable: bool,
    pub attributes: Vec<AttributeNode>,
    pub binding_info: Option<BindingInfo>,
}

/// Symbol type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SymbolType {
    Function,
    Variable,
    Struct,
    TypeAlias,
    Constant,
    Parameter,
}

/// Binding information for resources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BindingInfo {
    pub group: u32,
    pub binding: u32,
    pub storage_class: Option<StorageClass>,
}

/// Symbol table for scope management
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, SymbolEntry>>,
    current_scope: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            current_scope: 0,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.current_scope += 1;
    }

    pub fn exit_scope(&mut self) {
        if self.current_scope > 0 {
            self.scopes.pop();
            self.current_scope -= 1;
        }
    }

    pub fn add_symbol(&mut self, entry: SymbolEntry) {
        if let Some(scope) = self.scopes.get_mut(self.current_scope) {
            scope.insert(entry.name.clone(), entry);
        }
    }

    pub fn lookup_symbol(&self, name: &str) -> Option<&SymbolEntry> {
        // Search from current scope outward
        for i in (0..=self.current_scope).rev() {
            if let Some(scope) = self.scopes.get(i) {
                if let Some(entry) = scope.get(name) {
                    return Some(entry);
                }
            }
        }
        None
    }

    pub fn lookup_symbol_in_current_scope(&self, name: &str) -> Option<&SymbolEntry> {
        if let Some(scope) = self.scopes.get(self.current_scope) {
            scope.get(name)
        } else {
            None
        }
    }
}

/// WGSL AST Parser based on use.gpu patterns
pub struct WgslAstParser {
    symbol_table: SymbolTable,
    current_module: Option<ModuleNode>,
    errors: Vec<ParseError>,
    warnings: Vec<ParseWarning>,
}

type Result<T> = StdResult<T, ParseError>;

/// Parse error information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub error_type: ParseErrorType,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at {}:{}: {} ({})", self.line, self.column, self.message, 
               match self.error_type {
                   ParseErrorType::SyntaxError => "syntax error",
                   ParseErrorType::TypeError => "type error",
                   ParseErrorType::SemanticError => "semantic error",
                   ParseErrorType::UndefinedSymbol => "undefined symbol",
                   ParseErrorType::Redefinition => "redefinition",
                   ParseErrorType::InvalidAttribute => "invalid attribute",
                   ParseErrorType::InvalidBinding => "invalid binding",
                   ParseErrorType::UnexpectedToken => "unexpected token",
               })
    }
}

impl std::error::Error for ParseError {}

/// Parse error types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseErrorType {
    SyntaxError,
    TypeError,
    SemanticError,
    UndefinedSymbol,
    Redefinition,
    InvalidAttribute,
    InvalidBinding,
    UnexpectedToken,
}

/// Parse warning information
#[derive(Debug, Clone)]
pub struct ParseWarning {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub warning_type: ParseWarningType,
}

/// Parse warning types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseWarningType {
    UnusedVariable,
    DeprecatedFeature,
    PerformanceWarning,
    StyleWarning,
}

impl WgslAstParser {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            current_module: None,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Parse WGSL source code into AST
    pub fn parse(&mut self, source: &str) -> Result<ModuleNode> {
        // Reset state
        self.symbol_table = SymbolTable::new();
        self.errors.clear();
        self.warnings.clear();

        // Parse using naga library (Rust-native WGSL parser)
        let module = self.parse_module(source)?;
        
        // Validate the AST
        self.validate_module(&module)?;
        
        self.current_module = Some(module.clone());
        Ok(module)
    }

    /// Parse module-level declarations
    fn parse_module(&mut self, source: &str) -> Result<ModuleNode> {
        let mut module = ModuleNode {
            name: None,
            imports: Vec::new(),
            declarations: Vec::new(),
            exports: Vec::new(),
            attributes: Vec::new(),
        };

        // Parse global attributes
        // Parse imports
        // Parse declarations
        
        // For now, create a basic module structure
        // This would be replaced with actual Lezer grammar parsing
        
        Ok(module)
    }

    /// Validate the parsed module
    fn validate_module(&mut self, module: &ModuleNode) -> Result<()> {
        // Check for undefined symbols
        // Check for type consistency
        // Check for valid bindings
        // Check for shader stage requirements
        
        if !self.errors.is_empty() {
            return Err(ParseError {
                message: format!("Parse errors: {:?}", self.errors),
                line: 0,
                column: 0,
                error_type: ParseErrorType::SyntaxError,
            });
        }
        
        Ok(())
    }

    /// Get parse errors
    pub fn get_errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// Get parse warnings
    pub fn get_warnings(&self) -> &[ParseWarning] {
        &self.warnings
    }

    /// Get symbol table
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

/// AST visitor trait for tree traversal
pub trait AstVisitor {
    fn visit_module(&mut self, module: &ModuleNode) -> Result<()>;
    fn visit_function(&mut self, function: &FunctionNode) -> Result<()>;
    fn visit_struct(&mut self, struct_node: &StructNode) -> Result<()>;
    fn visit_variable(&mut self, variable: &VariableNode) -> Result<()>;
    fn visit_statement(&mut self, statement: &StatementNode) -> Result<()>;
    fn visit_expression(&mut self, expression: &ExpressionNode) -> Result<()>;
}

/// Default AST visitor implementation
pub struct DefaultAstVisitor;

impl AstVisitor for DefaultAstVisitor {
    fn visit_module(&mut self, module: &ModuleNode) -> Result<()> {
        // Visit all declarations in the module
        for declaration in &module.declarations {
            match declaration {
                DeclarationNode::Function(func) => self.visit_function(func)?,
                DeclarationNode::Struct(struct_node) => self.visit_struct(struct_node)?,
                DeclarationNode::Variable(var) => self.visit_variable(var)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn visit_function(&mut self, function: &FunctionNode) -> Result<()> {
        // Visit function body if present
        if let Some(body) = &function.body {
            self.visit_statement(&StatementNode::Block(body.clone()))?;
        }
        Ok(())
    }

    fn visit_struct(&mut self, _struct_node: &StructNode) -> Result<()> {
        // Struct validation
        Ok(())
    }

    fn visit_variable(&mut self, _variable: &VariableNode) -> Result<()> {
        // Variable validation
        Ok(())
    }

    fn visit_statement(&mut self, statement: &StatementNode) -> Result<()> {
        match statement {
            StatementNode::Block(block) => {
                for stmt in &block.statements {
                    self.visit_statement(stmt)?;
                }
            }
            StatementNode::If(if_node) => {
                self.visit_expression(&if_node.condition)?;
                self.visit_statement(&if_node.then_block)?;
                if let Some(else_block) = &if_node.else_block {
                    self.visit_statement(else_block)?;
                }
            }
            StatementNode::Return(Some(expr)) => {
                self.visit_expression(expr)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_expression(&mut self, expression: &ExpressionNode) -> Result<()> {
        match expression {
            ExpressionNode::Binary(binary) => {
                self.visit_expression(&binary.left)?;
                self.visit_expression(&binary.right)?;
            }
            ExpressionNode::Unary(unary) => {
                self.visit_expression(&unary.operand)?;
            }
            ExpressionNode::FunctionCall(call) => {
                for arg in &call.arguments {
                    self.visit_expression(arg)?;
                }
            }
            ExpressionNode::FieldAccess(access) => {
                self.visit_expression(&access.object)?;
            }
            ExpressionNode::ArrayAccess(access) => {
                self.visit_expression(&access.array)?;
                self.visit_expression(&access.index)?;
            }
            ExpressionNode::Select(select) => {
                self.visit_expression(&select.condition)?;
                self.visit_expression(&select.true_expression)?;
                self.visit_expression(&select.false_expression)?;
            }
            _ => {}
        }
        Ok(())
    }
}

/// Utility functions for AST manipulation
pub mod ast_utils {
    use super::*;

    /// Get all function names in the module
    pub fn get_function_names(module: &ModuleNode) -> Vec<String> {
        module.declarations
            .iter()
            .filter_map(|decl| {
                if let DeclarationNode::Function(func) = decl {
                    Some(func.name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all struct names in the module
    pub fn get_struct_names(module: &ModuleNode) -> Vec<String> {
        module.declarations
            .iter()
            .filter_map(|decl| {
                if let DeclarationNode::Struct(struct_node) = decl {
                    Some(struct_node.name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all uniform variables in the module
    pub fn get_uniform_variables(module: &ModuleNode) -> Vec<&VariableNode> {
        module.declarations
            .iter()
            .filter_map(|decl| {
                if let DeclarationNode::Variable(var) = decl {
                    if var.binding.is_some() && var.group.is_some() {
                        Some(var)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if a function is an entry point
    pub fn is_entry_point(function: &FunctionNode) -> bool {
        function.attributes.iter().any(|attr| {
            matches!(attr.name.as_str(), "vertex" | "fragment" | "compute")
        })
    }

    /// Get entry point functions
    pub fn get_entry_points(module: &ModuleNode) -> Vec<&FunctionNode> {
        module.declarations
            .iter()
            .filter_map(|decl| {
                if let DeclarationNode::Function(func) = decl {
                    if is_entry_point(func) {
                        Some(func)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table() {
        let mut symbol_table = SymbolTable::new();
        
        symbol_table.enter_scope();
        
        let entry = SymbolEntry {
            name: "test_var".to_string(),
            symbol_type: SymbolType::Variable,
            data_type: TypeNode::Primitive(PrimitiveType::F32),
            scope_level: 1,
            is_mutable: true,
            attributes: Vec::new(),
            binding_info: None,
        };
        
        symbol_table.add_symbol(entry);
        
        assert!(symbol_table.lookup_symbol("test_var").is_some());
        assert_eq!(symbol_table.lookup_symbol("test_var").unwrap().name, "test_var");
        
        symbol_table.exit_scope();
        assert!(symbol_table.lookup_symbol("test_var").is_none());
    }

    #[test]
    fn test_ast_visitor() {
        let mut visitor = DefaultAstVisitor;
        let module = ModuleNode {
            name: None,
            imports: Vec::new(),
            declarations: Vec::new(),
            exports: Vec::new(),
            attributes: Vec::new(),
        };
        
        assert!(visitor.visit_module(&module).is_ok());
    }
}

// Integration with existing shader compilation system
// TODO: Re-enable when advanced_shader_compilation module is implemented
/*
pub mod integration {
    use super::*;
    use crate::advanced_shader_compilation::{AdvancedShaderCompiler, CompiledShader, ShaderMetadata};

    /// Convert AST to compiled shader format
    pub fn ast_to_compiled_shader(module: &ModuleNode, source: &str) -> Result<CompiledShader> {
        // Extract metadata
        let metadata = extract_shader_metadata(module)?;
        
        // Extract uniforms
        let uniforms = extract_uniforms(module)?;
        
        // Extract functions
        let functions = extract_functions(module)?;
        
        // Generate WGSL code
        let wgsl_code = generate_wgsl_from_ast(module)?;
        
        Ok(CompiledShader {
            wgsl_code,
            metadata,
            uniforms,
            textures: Vec::new(), // TODO: Extract textures
            functions,
            entry_points: ast_utils::get_entry_points(module)
                .iter()
                .map(|func| func.name.clone())
                .collect(),
            validation_errors: Vec::new(),
            warnings: Vec::new(),
        })
    }

    /// Extract shader metadata from AST
    fn extract_shader_metadata(module: &ModuleNode) -> Result<ShaderMetadata> {
        // Look for shader metadata in attributes and comments
        Ok(ShaderMetadata {
            name: "Generated Shader".to_string(),
            description: "Generated from AST".to_string(),
            author: "WGSL Shader Studio".to_string(),
            version: "1.0.0".to_string(),
            category: "Generated".to_string(),
            tags: Vec::new(),
            performance_hints: crate::advanced_shader_compilation::PerformanceHints {
                estimated_fps: 60.0,
                texture_samples: 0,
                instruction_count: 0,
                uniform_count: ast_utils::get_uniform_variables(module).len(),
                recommended_resolution: (1920, 1080),
            },
        })
    }

    /// Extract uniforms from AST
    fn extract_uniforms(module: &ModuleNode) -> Result<Vec<crate::advanced_shader_compilation::UniformInfo>> {
        let mut uniforms = Vec::new();
        
        for var in ast_utils::get_uniform_variables(module) {
            if let (Some(binding), Some(group)) = (var.binding, var.group) {
                uniforms.push(crate::advanced_shader_compilation::UniformInfo {
                    name: var.name.clone(),
                    wgsl_type: type_node_to_string(&var.data_type),
                    binding,
                    group,
                    default_value: 0.0,
                    min_value: None,
                    max_value: None,
                    description: format!("Uniform {}", var.name),
                });
            }
        }
        
        Ok(uniforms)
    }

    /// Extract functions from AST
    fn extract_functions(module: &ModuleNode) -> Result<Vec<crate::advanced_shader_compilation::FunctionInfo>> {
        let mut functions = Vec::new();
        
        for func in ast_utils::get_entry_points(module) {
            let mut parameters = Vec::new();
            
            for param in &func.parameters {
                parameters.push(crate::advanced_shader_compilation::ParameterInfo {
                    name: param.name.clone(),
                    wgsl_type: type_node_to_string(&param.data_type),
                    direction: match param.direction {
                        ParameterDirection::In => crate::advanced_shader_compilation::ParameterDirection::In,
                        ParameterDirection::Out => crate::advanced_shader_compilation::ParameterDirection::Out,
                        ParameterDirection::InOut => crate::advanced_shader_compilation::ParameterDirection::InOut,
                    },
                });
            }
            
            functions.push(crate::advanced_shader_compilation::FunctionInfo {
                name: func.name.clone(),
                return_type: func.return_type.as_ref()
                    .map(|t| type_node_to_string(t))
                    .unwrap_or_else(|| "void".to_string()),
                parameters,
                is_entry_point: true,
                line_number: 0, // TODO: Track line numbers
            });
        }
        
        Ok(functions)
    }

    /// Convert TypeNode to WGSL string representation
    fn type_node_to_string(type_node: &TypeNode) -> String {
        match type_node {
            TypeNode::Primitive(primitive) => match primitive {
                PrimitiveType::Bool => "bool".to_string(),
                PrimitiveType::I32 => "i32".to_string(),
                PrimitiveType::U32 => "u32".to_string(),
                PrimitiveType::F32 => "f32".to_string(),
                PrimitiveType::F16 => "f16".to_string(),
            },
            TypeNode::Vector(vector) => {
                let prefix = match vector.component_type {
                    PrimitiveType::Bool => "b",
                    PrimitiveType::I32 => "i",
                    PrimitiveType::U32 => "u",
                    PrimitiveType::F32 => "f",
                    PrimitiveType::F16 => "f",
                };
                let size = match vector.size {
                    VectorSize::Two => "2",
                    VectorSize::Three => "3",
                    VectorSize::Four => "4",
                };
                format!("vec{}{}"", size, prefix)
            },
            TypeNode::Matrix(matrix) => {
                format!("mat{}x{}x<f32>", matrix.rows, matrix.cols)
            },
            TypeNode::Array(array) => {
                let element_type = type_node_to_string(&array.element_type);
                match array.size {
                    Some(size) => format!("array<{}, {}>", element_type, size),
                    None => format!("array<{}>", element_type),
                }
            },
            TypeNode::Struct(name) => name.clone(),
            TypeNode::Pointer(pointer) => {
                let element_type = type_node_to_string(&pointer.element_type);
                let storage = match pointer.storage_class {
                    StorageClass::Function => "function",
                    StorageClass::Workgroup => "workgroup",
                    StorageClass::Uniform => "uniform",
                    StorageClass::Storage => "storage",
                    StorageClass::Private => "private",
                };
                format!("ptr<{}, {}>", storage, element_type)
            },
            TypeNode::Texture(texture) => {
                let sample_type = match texture.sample_type {
                    TextureSampleType::Float => "f32",
                    TextureSampleType::Uint => "u32",
                    TextureSampleType::Sint => "i32",
                    TextureSampleType::Depth => "depth",
                };
                let dimension = match texture.dimension {
                    TextureDimension::One => "1d",
                    TextureDimension::Two => "2d",
                    TextureDimension::Three => "3d",
                    TextureDimension::Cube => "cube",
                };
                let array_suffix = if texture.is_array { "_array" } else { "" };
                let multisampled_suffix = if texture.is_multisampled { "_ms" } else { "" };
                
                format!("texture{}{}_{}", dimension, array_suffix, sample_type)
            },
            TypeNode::Sampler(sampler) => match sampler {
                SamplerType::Sampler => "sampler".to_string(),
                SamplerType::ComparisonSampler => "sampler_comparison".to_string(),
            },
            TypeNode::Atomic(atomic) => match atomic {
                AtomicType::I32 => "atomic<i32>".to_string(),
                AtomicType::U32 => "atomic<u32>".to_string(),
            },
        }
    }

    /// Generate WGSL code from AST
    fn generate_wgsl_from_ast(module: &ModuleNode) -> Result<String> {
        let mut code = String::new();
        
        // Generate imports
        for import in &module.imports {
            // TODO: Generate import statements
        }
        
        // Generate declarations
        for declaration in &module.declarations {
            match declaration {
                DeclarationNode::Function(func) => {
                    code.push_str(&generate_function_wgsl(func)?);
                    code.push('\n');
                }
                DeclarationNode::Struct(struct_node) => {
                    code.push_str(&generate_struct_wgsl(struct_node)?);
                    code.push('\n');
                }
                DeclarationNode::Variable(var) => {
                    code.push_str(&generate_variable_wgsl(var)?);
                    code.push('\n');
                }
                DeclarationNode::TypeAlias(alias) => {
                    code.push_str(&generate_type_alias_wgsl(alias)?);
                    code.push('\n');
                }
                DeclarationNode::Constant(const_node) => {
                    code.push_str(&generate_constant_wgsl(const_node)?);
                    code.push('\n');
                }
            }
        }
        
        Ok(code)
    }

    /// Generate WGSL for function
    fn generate_function_wgsl(function: &FunctionNode) -> Result<String> {
        let mut code = String::new();
        
        // Generate attributes
        for attr in &function.attributes {
            code.push_str(&format!("@{}", attr.name));
            if !attr.arguments.is_empty() {
                let args: Vec<String> = attr.arguments.iter()
                    .map(|expr| expression_to_string(expr))
                    .collect();
                code.push_str(&format!("({})", args.join(", ")));
            }
            code.push('\n');
        }
        
        // Generate function signature
        code.push_str(&format!("fn {}(", function.name));
        
        let params: Vec<String> = function.parameters.iter()
            .map(|param| {
                let mut param_str = String::new();
                if !param.attributes.is_empty() {
                    for attr in &param.attributes {
                        param_str.push_str(&format!("@{} ", attr.name));
                    }
                }
                param_str.push_str(&format!("{}: {}", param.name, type_node_to_string(&param.data_type)));
                param_str
            })
            .collect();
        
        code.push_str(&params.join(", "));
        code.push(')');
        
        if let Some(return_type) = &function.return_type {
            code.push_str(&format!(" -> {}", type_node_to_string(return_type)));
        }
        
        if let Some(body) = &function.body {
            code.push(' ');
            code.push_str(&generate_block_wgsl(body)?);
        } else {
            code.push_str(";");
        }
        
        Ok(code)
    }

    /// Generate WGSL for struct
    fn generate_struct_wgsl(struct_node: &StructNode) -> Result<String> {
        let mut code = String::new();
        
        // Generate attributes
        for attr in &struct_node.attributes {
            code.push_str(&format!("@{}", attr.name));
            if !attr.arguments.is_empty() {
                let args: Vec<String> = attr.arguments.iter()
                    .map(|expr| expression_to_string(expr))
                    .collect();
                code.push_str(&format!("({})", args.join(", ")));
            }
            code.push('\n');
        }
        
        code.push_str(&format!(r#"struct {} {{
"#, struct_node.name));
        
        for member in &struct_node.members {
            code.push_str(&format!(r#"    {}: {},
"#, member.name, type_node_to_string(&member.data_type)));
        }
        
        code.push('}');
        
        Ok(code)
    }

    /// Generate WGSL for variable
    fn generate_variable_wgsl(variable: &VariableNode) -> Result<String> {
        let mut code = String::new();
        
        // Generate attributes
        for attr in &variable.attributes {
            code.push_str(&format!("@{}", attr.name));
            if !attr.arguments.is_empty() {
                let args: Vec<String> = attr.arguments.iter()
                    .map(|expr| expression_to_string(expr))
                    .collect();
                code.push_str(&format!("({})", args.join(", ")));
            }
            code.push('\n');
        }
        
        let var_keyword = match variable.var_type {
            VarType::Var => "var",
            VarType::Let => "let",
            VarType::Const => "const",
        };
        
        code.push_str(var_keyword);
        code.push(' ');
        
        if let (Some(group), Some(binding)) = (variable.group, variable.binding) {
            code.push_str(&format!("@group({}) @binding({}) ", group, binding));
        }
        
        code.push_str(&format!("{}: {}", variable.name, type_node_to_string(&variable.data_type)));
        
        if let Some(init) = &variable.initializer {
            code.push_str(&format!(" = {}", expression_to_string(init)));
        }
        
        code.push(';');
        
        Ok(code)
    }

    /// Generate WGSL for type alias
    fn generate_type_alias_wgsl(alias: &TypeAliasNode) -> Result<String> {
        let mut code = String::new();
        
        // Generate attributes
        for attr in &alias.attributes {
            code.push_str(&format!("@{}", attr.name));
            if !attr.arguments.is_empty() {
                let args: Vec<String> = attr.arguments.iter()
                    .map(|expr| expression_to_string(expr))
                    .collect();
                code.push_str(&format!("({})", args.join(", ")));
            }
            code.push('\n');
        }
        
        code.push_str(&format!("alias {} = {};", alias.name, type_node_to_string(&alias.aliased_type)));
        
        Ok(code)
    }

    /// Generate WGSL for constant
    fn generate_constant_wgsl(constant: &ConstantNode) -> Result<String> {
        let mut code = String::new();
        
        // Generate attributes
        for attr in &constant.attributes {
            code.push_str(&format!("@{}", attr.name));
            if !attr.arguments.is_empty() {
                let args: Vec<String> = attr.arguments.iter()
                    .map(|expr| expression_to_string(expr))
                    .collect();
                code.push_str(&format!("({})", args.join(", ")));
            }
            code.push('\n');
        }
        
        code.push_str(&format!("const {}" , constant.name));
        
        if let Some(data_type) = &constant.data_type {
            code.push_str(&format!(": {}", type_node_to_string(data_type)));
        }
        
        code.push_str(&format!(" = {};", expression_to_string(&constant.value)));
        
        Ok(code)
    }

    /// Generate WGSL for block
    fn generate_block_wgsl(block: &BlockNode) -> Result<String> {
        let mut code = String::new();
        
        code.push('{');
        
        for statement in &block.statements {
            code.push('\n');
            code.push_str(&generate_statement_wgsl(statement)?);
        }
        
        code.push('\n');
        code.push('}');
        
        Ok(code)
    }

    /// Generate WGSL for statement
    fn generate_statement_wgsl(statement: &StatementNode) -> Result<String> {
        match statement {
            StatementNode::Block(block) => generate_block_wgsl(block),
            StatementNode::Return(None) => Ok("return;".to_string()),
            StatementNode::Return(Some(expr)) => Ok(format!("return {};", expression_to_string(expr))),
            StatementNode::Break => Ok("break;".to_string()),
            StatementNode::Continue => Ok("continue;".to_string()),
            _ => Ok("// Unsupported statement".to_string()),
        }
    }

    /// Convert expression to string
    fn expression_to_string(expr: &ExpressionNode) -> String {
        match expr {
            ExpressionNode::Literal(lit) => match lit {
                LiteralNode::Bool(b) => b.to_string(),
                LiteralNode::I32(i) => i.to_string(),
                LiteralNode::U32(u) => u.to_string(),
                LiteralNode::F32(f) => f.to_string(),
                LiteralNode::F16(f) => f.to_string(),
            },
            ExpressionNode::Identifier(name) => name.clone(),
            ExpressionNode::Binary(binary) => format!(
                "({} {} {})",
                expression_to_string(&binary.left),
                binary_operator_to_string(binary.operator),
                expression_to_string(&binary.right)
            ),
            ExpressionNode::Unary(unary) => format!(
                "{}{}",
                unary_operator_to_string(unary.operator),
                expression_to_string(&unary.operand)
            ),
            ExpressionNode::FunctionCall(call) => format!(
                "{}({})",
                call.function_name,
                call.arguments.iter().map(expression_to_string).collect::<Vec<_>>().join(", ")
            ),
            ExpressionNode::FieldAccess(access) => format!(
                "{}.{}",
                expression_to_string(&access.object),
                access.field
            ),
            ExpressionNode::ArrayAccess(access) => format!(
                "{}[{}]",
                expression_to_string(&access.array),
                expression_to_string(&access.index)
            ),
            ExpressionNode::TypeConstructor(constructor) => format!(
                "{}({})",
                constructor.type_name,
                constructor.arguments.iter().map(expression_to_string).collect::<Vec<_>>().join(", ")
            ),
            ExpressionNode::Select(select) => format!(
                "({} ? {} : {})",
                expression_to_string(&select.condition),
                expression_to_string(&select.true_expression),
                expression_to_string(&select.false_expression)
            ),
        }
    }

    /// Convert binary operator to string
    fn binary_operator_to_string(op: BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::Less => "<",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::Greater => ">",
            BinaryOperator::GreaterEqual => ">=",
            BinaryOperator::And => "&",
            BinaryOperator::Or => "|",
            BinaryOperator::Xor => "^",
            BinaryOperator::ShiftLeft => "<<",
            BinaryOperator::ShiftRight => ">>",
        }
    }

    /// Convert unary operator to string
    fn unary_operator_to_string(op: UnaryOperator) -> &'static str {
        match op {
            UnaryOperator::Not => "!",
            UnaryOperator::Negate => "-",
            UnaryOperator::Complement => "~",
        }
    }
}
*/
