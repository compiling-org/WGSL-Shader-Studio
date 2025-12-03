#!/usr/bin/env python3
"""
WGSL Shader Studio - Comprehensive UI Analyzer Test
This script tests the UI analyzer functionality without cargo compilation issues.
"""

import os
import sys
import json
import subprocess
import time
from pathlib import Path

def analyze_project_structure():
    """Analyze the project structure and identify key files"""
    print("🔍 WGSL Shader Studio - Comprehensive UI Analyzer")
    print("=" * 60)
    
    project_root = Path(".")
    
    # Check for key files
    key_files = {
        "src/lib.rs": "Library main file",
        "src/editor_ui.rs": "Editor UI implementation",
        "src/ui_analyzer.rs": "UI analyzer module",
        "src/ui_analyzer_enhanced.rs": "Enhanced UI analyzer",
        "Cargo.toml": "Project configuration",
        "comprehensive_enforcer_enhanced.ps1": "Enforcer script",
        ".trae/documents/comprehensive_work_documentation.md": "Documentation"
    }
    
    print("📁 Project Structure Analysis:")
    print("-" * 40)
    
    missing_files = []
    present_files = []
    
    for file_path, description in key_files.items():
        full_path = project_root / file_path
        if full_path.exists():
            size = full_path.stat().st_size
            present_files.append((file_path, description, size))
            print(f"✅ {file_path} ({description}) - {size} bytes")
        else:
            missing_files.append((file_path, description))
            print(f"❌ {file_path} ({description}) - MISSING")
    
    print(f"\n📊 Summary:")
    print(f"  Present files: {len(present_files)}")
    print(f"  Missing files: {len(missing_files)}")
    
    return present_files, missing_files

def analyze_cargo_toml():
    """Analyze Cargo.toml for dependencies and features"""
    print("\n🔧 Cargo.toml Analysis:")
    print("-" * 40)
    
    try:
        with open("Cargo.toml", "r") as f:
            content = f.read()
        
        # Check for key dependencies
        key_deps = [
            "bevy", "wgpu", "egui", "tokio", "midir", "rustfft", 
            "serde", "regex", "naga", "image", "rfd"
        ]
        
        found_deps = []
        missing_deps = []
        
        for dep in key_deps:
            if dep in content:
                found_deps.append(dep)
            else:
                missing_deps.append(dep)
        
        print(f"✅ Found dependencies: {', '.join(found_deps)}")
        if missing_deps:
            print(f"⚠️  Missing dependencies: {', '.join(missing_deps)}")
        
        # Check for features
        if "[features]" in content:
            print("✅ Features section found")
        else:
            print("⚠️  No features section found")
            
        # Check for binaries
        if "[[bin]]" in content:
            print("✅ Binary targets found")
        else:
            print("⚠️  No binary targets found")
            
    except Exception as e:
        print(f"❌ Error analyzing Cargo.toml: {e}")

def analyze_source_files():
    """Analyze source files for key functionality"""
    print("\n📄 Source File Analysis:")
    print("-" * 40)
    
    source_files = [
        "src/lib.rs",
        "src/editor_ui.rs", 
        "src/ui_analyzer.rs",
        "src/ui_analyzer_enhanced.rs"
    ]
    
    for file_path in source_files:
        if os.path.exists(file_path):
            try:
                with open(file_path, "r") as f:
                    content = f.read()
                
                lines = len(content.split('\n'))
                
                # Check for key patterns
                patterns = {
                    "UI Analyzer": "UIAnalyzer|ui_analyzer",
                    "Editor UI": "EditorUi|editor_ui", 
                    "Shader Parameters": "ShaderParameter|parse_shader_parameters",
                    "WGSL Support": "wgsl|WGSL",
                    "ISF Support": "isf|ISF",
                    "Audio Integration": "audio|midi|Audio|MIDI",
                    "GPU Features": "gpu|GPU|wgpu|WGPU",
                    "Module System": "module_system|ModuleId"
                }
                
                found_patterns = []
                for name, pattern in patterns.items():
                    import re
                    if re.search(pattern, content, re.IGNORECASE):
                        found_patterns.append(name)
                
                print(f"📄 {file_path} ({lines} lines)")
                if found_patterns:
                    print(f"   Found: {', '.join(found_patterns)}")
                else:
                    print("   No key patterns detected")