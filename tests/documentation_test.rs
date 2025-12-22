//! Test to verify that all documentation files exist and are properly linked

use std::fs;
use std::path::Path;

#[test]
fn test_documentation_files_exist() {
    let docs_dir = Path::new("docs");
    
    // Check that the docs directory exists
    assert!(docs_dir.exists(), "Docs directory should exist");
    assert!(docs_dir.is_dir(), "Docs should be a directory");
    
    // List of expected documentation files
    let expected_files = vec![
        "WGSL_FUNDAMENTALS.md",
        "GLSL_FUNDAMENTALS.md",
        "HLSL_FUNDAMENTALS.md",
        "ISF_FUNDAMENTALS.md",
        "SHADER_CONVERSION_FRAMEWORK.md",
        "APPLICATION_USAGE_GUIDE_COMPLETE.md",
        "WGSL_SHADER_STUDIO_ARCHITECTURE.md",
        "ADVANCED_FEATURES.md",
        "COMPREHENSIVE_DOCUMENTATION_INDEX.md",
        "SHADER_STUDIO_COOKBOOK.md",
        "index.html",
    ];
    
    // Check that each file exists
    for file_name in expected_files {
        let file_path = docs_dir.join(file_name);
        assert!(file_path.exists(), "Documentation file {} should exist", file_name);
        assert!(file_path.is_file(), "Documentation file {} should be a file", file_name);
        
        // Check that the file is not empty
        let metadata = fs::metadata(&file_path).expect("Should be able to get file metadata");
        assert!(metadata.len() > 0, "Documentation file {} should not be empty", file_name);
    }
    
    println!("All documentation files exist and are not empty");
}

#[test]
fn test_readme_links() {
    let readme_path = Path::new("README.md");
    assert!(readme_path.exists(), "README.md should exist");
    
    let readme_content = fs::read_to_string(readme_path).expect("Should be able to read README.md");
    
    // Check that README contains links to the new documentation
    assert!(readme_content.contains("WGSL Fundamentals"), "README should link to WGSL Fundamentals");
    assert!(readme_content.contains("GLSL Fundamentals"), "README should link to GLSL Fundamentals");
    assert!(readme_content.contains("HLSL Fundamentals"), "README should link to HLSL Fundamentals");
    assert!(readme_content.contains("ISF Fundamentals"), "README should link to ISF Fundamentals");
    assert!(readme_content.contains("Shader Conversion Framework"), "README should link to Shader Conversion Framework");
    assert!(readme_content.contains("Application Usage Guide"), "README should link to Application Usage Guide");
    assert!(readme_content.contains("Technical Architecture"), "README should link to Technical Architecture");
    assert!(readme_content.contains("Advanced Features"), "README should link to Advanced Features");
    assert!(readme_content.contains("Comprehensive Documentation Index"), "README should link to Comprehensive Documentation Index");
    
    println!("README.md contains links to all documentation files");
}

#[test]
fn test_documentation_index_links() {
    let index_path = Path::new("docs/COMPREHENSIVE_DOCUMENTATION_INDEX.md");
    assert!(index_path.exists(), "COMPREHENSIVE_DOCUMENTATION_INDEX.md should exist");
    
    let index_content = fs::read_to_string(index_path).expect("Should be able to read COMPREHENSIVE_DOCUMENTATION_INDEX.md");
    
    // Check that the index contains links to all documentation files
    assert!(index_content.contains("./WGSL_FUNDAMENTALS.md"), "Index should link to WGSL_FUNDAMENTALS.md");
    assert!(index_content.contains("./GLSL_FUNDAMENTALS.md"), "Index should link to GLSL_FUNDAMENTALS.md");
    assert!(index_content.contains("./HLSL_FUNDAMENTALS.md"), "Index should link to HLSL_FUNDAMENTALS.md");
    assert!(index_content.contains("./ISF_FUNDAMENTALS.md"), "Index should link to ISF_FUNDAMENTALS.md");
    assert!(index_content.contains("./SHADER_CONVERSION_FRAMEWORK.md"), "Index should link to SHADER_CONVERSION_FRAMEWORK.md");
    assert!(index_content.contains("./APPLICATION_USAGE_GUIDE_COMPLETE.md"), "Index should link to APPLICATION_USAGE_GUIDE_COMPLETE.md");
    assert!(index_content.contains("./WGSL_SHADER_STUDIO_ARCHITECTURE.md"), "Index should link to WGSL_SHADER_STUDIO_ARCHITECTURE.md");
    assert!(index_content.contains("./ADVANCED_FEATURES.md"), "Index should link to ADVANCED_FEATURES.md");
    
    println!("Documentation index contains links to all documentation files");
}