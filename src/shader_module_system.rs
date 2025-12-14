use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::fmt;

use crate::wgsl_ast_parser::{AstNode, WgslAstParser, ParseError, ModuleNode};
use crate::advanced_shader_compilation::{CompiledShader, ShaderCompilationError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub String);

impl fmt::Display for ModuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderModule {
    pub id: ModuleId,
    pub name: String,
    pub source: String,
    #[serde(skip)]
    pub ast: Option<AstNode>,
    #[serde(skip)]
    pub compiled: Option<CompiledShader>,
    pub dependencies: HashSet<ModuleId>,
    pub exports: HashSet<String>,
    pub imports: HashSet<String>,
    #[serde(skip_serializing, skip_deserializing, default = "default_instant")]
    pub last_modified: Instant,
    pub version: u64,
}

fn default_instant() -> Instant {
    Instant::now()
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleBundle {
    pub name: String,
    pub modules: HashMap<ModuleId, ShaderModule>,
    pub entry_points: HashSet<ModuleId>,
    pub metadata: BundleMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub tags: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleCacheEntry {
    pub module: Arc<ShaderModule>,
    pub last_accessed: Instant,
    pub access_count: u64,
}

#[derive(Debug, Clone)]
pub struct ImportResolution {
    pub resolved_id: ModuleId,
    pub import_path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Error)]
pub enum ModuleSystemError {
    #[error("Module not found: {0:?}")]
    ModuleNotFound(ModuleId),
    
    #[error("Circular dependency detected: {0:?}")]
    CircularDependency(Vec<ModuleId>),
    
    #[error("Import resolution failed: {0}")]
    ImportResolutionFailed(String),
    
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    
    #[error("Compilation error: {0}")]
    CompilationError(#[from] ShaderCompilationError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Cache error: {0}")]
    CacheError(String),
}

pub type ModuleResult<T> = Result<T, ModuleSystemError>;

pub struct ShaderModuleSystem {
    modules: Arc<RwLock<HashMap<ModuleId, Arc<ShaderModule>>>>,
    cache: Arc<RwLock<LruCache<ModuleId, ModuleCacheEntry>>>,
    import_resolver: Arc<ImportResolver>,
    bundle_loader: Arc<BundleLoader>,
    parser: Arc<RwLock<WgslAstParser>>,
    max_cache_size: usize,
    cache_ttl: Duration,
}

impl ShaderModuleSystem {
    pub fn new(max_cache_size: usize, cache_ttl: Duration) -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(LruCache::new(std::num::NonZeroUsize::new(max_cache_size).unwrap()))),
            import_resolver: Arc::new(ImportResolver::new()),
            bundle_loader: Arc::new(BundleLoader::new()),
            parser: Arc::new(RwLock::new(WgslAstParser::new())),
            max_cache_size,
            cache_ttl,
        }
    }

    pub fn load_module(&self, id: ModuleId, source: String) -> ModuleResult<Arc<ShaderModule>> {
        if let Some(cached) = self.get_cached(&id)? {
            return Ok(cached);
        }

        let mut module = ShaderModule {
            id: id.clone(),
            name: id.0.clone(),
            source: source.clone(),
            ast: None,
            compiled: None,
            dependencies: HashSet::new(),
            exports: HashSet::new(),
            imports: HashSet::new(),
            last_modified: Instant::now(),
            version: 1,
        };

        let module_node = {
            let mut parser = self.parser.write().unwrap();
            parser.parse(&source)?
        };
        self.extract_module_info(&mut module, &module_node)?;
        module.ast = Some(AstNode::Module(module_node.clone()));

        let module = Arc::new(module);
        self.insert_module(id.clone(), module.clone())?;
        self.cache_module(id, module.clone())?;

        Ok(module)
    }

    pub fn load_module_from_file(&self, path: &Path) -> ModuleResult<Arc<ShaderModule>> {
        let source = std::fs::read_to_string(path)?;
        let id = ModuleId(path.file_stem().unwrap().to_string_lossy().to_string());
        self.load_module(id, source)
    }

    pub fn load_bundle(&self, bundle_path: &Path) -> ModuleResult<ModuleBundle> {
        self.bundle_loader.load_bundle(bundle_path)
    }

    pub fn resolve_imports(&self, module: &ShaderModule) -> ModuleResult<Vec<ImportResolution>> {
        self.import_resolver.resolve_imports(module)
    }

    pub fn compile_module(&self, id: &ModuleId) -> ModuleResult<CompiledShader> {
        let module = self.get_module(id)?;
        
        if let Some(compiled) = &module.compiled {
            return Ok(compiled.clone());
        }

        let dependencies = self.resolve_dependencies(id)?;
        let compiled = self.compile_with_dependencies(&module, &dependencies)?;

        let mut module_mut = (*module).clone();
        module_mut.compiled = Some(compiled.clone());
        let updated_module = Arc::new(module_mut);
        
        self.insert_module(id.clone(), updated_module)?;
        Ok(compiled)
    }

    pub fn get_module(&self, id: &ModuleId) -> ModuleResult<Arc<ShaderModule>> {
        if let Some(cached) = self.get_cached(id)? {
            return Ok(cached);
        }

        let modules = self.modules.read().unwrap();
        modules.get(id)
            .cloned()
            .ok_or_else(|| ModuleSystemError::ModuleNotFound(id.clone()))
    }

    pub fn invalidate_cache(&self, id: &ModuleId) -> ModuleResult<()> {
        let mut cache = self.cache.write().unwrap();
        cache.pop(id);
        Ok(())
    }

    pub fn clear_cache(&self) -> ModuleResult<()> {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
        Ok(())
    }

    pub fn get_cache_stats(&self) -> ModuleResult<CacheStats> {
        let cache = self.cache.read().unwrap();
        Ok(CacheStats {
            size: cache.len(),
            capacity: self.max_cache_size,
            hit_rate: 0.0,
            miss_rate: 0.0,
            eviction_count: 0,
        })
    }

    fn get_cached(&self, id: &ModuleId) -> ModuleResult<Option<Arc<ShaderModule>>> {
        let mut cache = self.cache.write().unwrap();
        
        if let Some(entry) = cache.get_mut(id) {
            if entry.last_accessed.elapsed() < self.cache_ttl {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                return Ok(Some(entry.module.clone()));
            } else {
                cache.pop(id);
            }
        }
        
        Ok(None)
    }

    fn cache_module(&self, id: ModuleId, module: Arc<ShaderModule>) -> ModuleResult<()> {
        let mut cache = self.cache.write().unwrap();
        let entry = ModuleCacheEntry {
            module,
            last_accessed: Instant::now(),
            access_count: 1,
        };
        cache.put(id, entry);
        Ok(())
    }

    fn insert_module(&self, id: ModuleId, module: Arc<ShaderModule>) -> ModuleResult<()> {
        let mut modules = self.modules.write().unwrap();
        modules.insert(id, module);
        Ok(())
    }

    fn extract_module_info(&self, module: &mut ShaderModule, module_node: &ModuleNode) -> ModuleResult<()> {
        for declaration in &module_node.declarations {
            self.extract_declaration_info(module, declaration)?;
        }
        Ok(())
    }

    fn extract_declaration_info(&self, module: &mut ShaderModule, declaration: &crate::wgsl_ast_parser::DeclarationNode) -> ModuleResult<()> {
        use crate::wgsl_ast_parser::DeclarationNode::*;
        match declaration {
            Function(func) => {
                module.exports.insert(func.name.clone());
            }
            Struct(strukt) => {
                module.exports.insert(strukt.name.clone());
            }
            Variable(var) => {
                module.exports.insert(var.name.clone());
            }
            TypeAlias(alias) => {
                module.exports.insert(alias.name.clone());
            }
            Constant(constant) => {
                module.exports.insert(constant.name.clone());
            }
        }
        Ok(())
    }

    fn extract_item_info(&self, module: &mut ShaderModule, item: &AstNode) -> ModuleResult<()> {
        match item {
            AstNode::FunctionDecl { name, .. } => {
                module.exports.insert(name.clone());
            }
            AstNode::StructDecl { name, .. } => {
                module.exports.insert(name.clone());
            }
            AstNode::ConstDecl { name, .. } => {
                module.exports.insert(name.clone());
            }
            AstNode::OverrideDecl { name, .. } => {
                module.exports.insert(name.clone());
            }
            AstNode::TypeAliasDecl { name, .. } => {
                module.exports.insert(name.clone());
            }
            AstNode::ImportDecl { path, .. } => {
                module.imports.insert(path.clone());
            }
            _ => {}
        }
        Ok(())
    }

    fn resolve_dependencies(&self, id: &ModuleId) -> ModuleResult<Vec<Arc<ShaderModule>>> {
        let module = self.get_module(id)?;
        let mut dependencies = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        self.collect_dependencies(&module, &mut dependencies, &mut visited, &mut stack)?;
        Ok(dependencies)
    }

    fn collect_dependencies(
        &self,
        module: &ShaderModule,
        dependencies: &mut Vec<Arc<ShaderModule>>,
        visited: &mut HashSet<ModuleId>,
        stack: &mut Vec<ModuleId>,
    ) -> ModuleResult<()> {
        if stack.contains(&module.id) {
            return Err(ModuleSystemError::CircularDependency(stack.clone()));
        }

        if visited.contains(&module.id) {
            return Ok(());
        }

        visited.insert(module.id.clone());
        stack.push(module.id.clone());

        for import in &module.imports {
            let import_id = ModuleId(import.clone());
            if let Ok(dep_module) = self.get_module(&import_id) {
                self.collect_dependencies(&dep_module, dependencies, visited, stack)?;
            }
        }

        stack.pop();
        dependencies.push(Arc::new(module.clone()));
        Ok(())
    }

    fn compile_with_dependencies(
        &self,
        module: &ShaderModule,
        dependencies: &[Arc<ShaderModule>],
    ) -> ModuleResult<CompiledShader> {
        let mut combined_source = String::new();
        
        for dep in dependencies {
            combined_source.push_str(&dep.source);
            combined_source.push('\n');
        }

        combined_source.push_str(&module.source);

        let compiled = crate::advanced_shader_compilation::AdvancedShaderCompiler::new()
            .compile(&combined_source)
            .map_err(|e| ModuleSystemError::CompilationError(
                crate::advanced_shader_compilation::ShaderCompilationError::ValidationError(e.to_string())
            ))?;

        Ok(compiled)
    }
}

pub struct ImportResolver {
    alias_map: Arc<RwLock<HashMap<String, ModuleId>>>,
}

impl ImportResolver {
    pub fn new() -> Self {
        Self {
            alias_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn resolve_imports(&self, module: &ShaderModule) -> ModuleResult<Vec<ImportResolution>> {
        let mut resolutions = Vec::new();
        
        for import in &module.imports {
            let resolved = self.resolve_single_import(import)?;
            resolutions.push(resolved);
        }

        Ok(resolutions)
    }

    pub fn register_alias(&self, alias: String, module_id: ModuleId) -> ModuleResult<()> {
        let mut alias_map = self.alias_map.write().unwrap();
        alias_map.insert(alias, module_id);
        Ok(())
    }

    fn resolve_single_import(&self, import_path: &str) -> ModuleResult<ImportResolution> {
        let alias_map = self.alias_map.read().unwrap();
        
        let resolved_id = if let Some(id) = alias_map.get(import_path) {
            id.clone()
        } else {
            ModuleId(import_path.to_string())
        };

        Ok(ImportResolution {
            resolved_id,
            import_path: import_path.to_string(),
            alias: None,
        })
    }
}

pub struct BundleLoader {
    supported_formats: Arc<RwLock<HashSet<String>>>,
}

impl BundleLoader {
    pub fn new() -> Self {
        let mut formats = HashSet::new();
        formats.insert("json".to_string());
        formats.insert("toml".to_string());
        formats.insert("yaml".to_string());

        Self {
            supported_formats: Arc::new(RwLock::new(formats)),
        }
    }

    pub fn load_bundle(&self, path: &Path) -> ModuleResult<ModuleBundle> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| ModuleSystemError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file extension")
            ))?;

        let content = std::fs::read_to_string(path)?;
        
        match extension {
            "json" => self.parse_json_bundle(&content),
            "toml" => self.parse_toml_bundle(&content),
            "yaml" | "yml" => self.parse_yaml_bundle(&content),
            _ => Err(ModuleSystemError::IoError(
                std::io::Error::new(std::io::ErrorKind::Unsupported, "Unsupported bundle format")
            )),
        }
    }

    fn parse_json_bundle(&self, content: &str) -> ModuleResult<ModuleBundle> {
        serde_json::from_str(content)
            .map_err(|e| ModuleSystemError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e)
            ))
    }

    fn parse_toml_bundle(&self, content: &str) -> ModuleResult<ModuleBundle> {
        toml::from_str(content)
            .map_err(|e| ModuleSystemError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e)
            ))
    }

    fn parse_yaml_bundle(&self, content: &str) -> ModuleResult<ModuleBundle> {
        serde_yaml::from_str(content)
            .map_err(|e| ModuleSystemError::IoError(
                std::io::Error::new(std::io::ErrorKind::InvalidData, e)
            ))
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub eviction_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loading() {
        let system = ShaderModuleSystem::new(100, Duration::from_secs(300));
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

        let result = system.load_module(ModuleId("test".to_string()), wgsl_source.to_string());
        assert!(result.is_ok());
        
        let module = result.unwrap();
        assert_eq!(module.name, "test");
        assert!(module.exports.contains("vs_main"));
        assert!(module.exports.contains("VertexInput"));
    }

    #[test]
    fn test_dependency_resolution() {
        let system = ShaderModuleSystem::new(100, Duration::from_secs(300));
        
        let math_module = r#"
            fn add(a: f32, b: f32) -> f32 {
                return a + b;
            }
        "#;

        let main_module = r#"
            import "math";
            
            @compute @workgroup_size(64)
            fn main() {
                let result = add(1.0, 2.0);
            }
        "#;

        system.load_module(ModuleId("math".to_string()), math_module.to_string()).unwrap();
        system.load_module(ModuleId("main".to_string()), main_module.to_string()).unwrap();

        let result = system.resolve_dependencies(&ModuleId("main".to_string()));
        assert!(result.is_ok());
        
        let deps = result.unwrap();
        assert_eq!(deps.len(), 2); // math + main
    }

    #[test]
    fn test_circular_dependency_detection() {
        let system = ShaderModuleSystem::new(100, Duration::from_secs(300));
        
        let module_a = r#"
            import "module_b";
            fn func_a() -> f32 { return 1.0; }
        "#;

        let module_b = r#"
            import "module_a";
            fn func_b() -> f32 { return 2.0; }
        "#;

        system.load_module(ModuleId("module_a".to_string()), module_a.to_string()).unwrap();
        system.load_module(ModuleId("module_b".to_string()), module_b.to_string()).unwrap();

        let result = system.resolve_dependencies(&ModuleId("module_a".to_string()));
        assert!(matches!(result, Err(ModuleSystemError::CircularDependency(_))));
    }

    #[test]
    fn test_cache_invalidation() {
        let system = ShaderModuleSystem::new(10, Duration::from_millis(100));
        let wgsl_source = "fn test() -> f32 { return 1.0; }";

        let module_id = ModuleId("cache_test".to_string());
        system.load_module(module_id.clone(), wgsl_source.to_string()).unwrap();
        
        let cached = system.get_cached(&module_id).unwrap();
        assert!(cached.is_some());

        std::thread::sleep(Duration::from_millis(150));
        
        let expired = system.get_cached(&module_id).unwrap();
        assert!(expired.is_none());
    }
}
