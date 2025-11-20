#!/usr/bin/env python3
"""
Mermaid Diagram Verification Script
Tests all mermaid diagrams in documentation files for syntax validity
"""

import re
import os
import json
import subprocess
from pathlib import Path

def extract_mermaid_diagrams(file_path):
    """Extract all mermaid diagrams from a markdown file"""
    diagrams = []
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Find all mermaid code blocks
        mermaid_pattern = r'```mermaid\n(.*?)\n```'
        matches = re.findall(mermaid_pattern, content, re.DOTALL)
        
        for i, match in enumerate(matches):
            diagrams.append({
                'id': f"{file_path.name}_{i}",
                'file': str(file_path),
                'content': match.strip(),
                'line_number': content[:content.find(match)].count('\n') + 1
            })
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
    
    return diagrams

def validate_mermaid_syntax(diagram_content):
    """Basic validation of mermaid diagram syntax"""
    errors = []
    
    # Check for basic syntax issues
    lines = diagram_content.split('\n')
    
    # Check for unclosed brackets
    open_brackets = diagram_content.count('[') - diagram_content.count(']')
    open_braces = diagram_content.count('{') - diagram_content.count('}')
    open_parens = diagram_content.count('(') - diagram_content.count(')')
    
    if open_brackets != 0:
        errors.append(f"Unclosed brackets: {open_brackets}")
    if open_braces != 0:
        errors.append(f"Unclosed braces: {open_braces}")
    if open_parens != 0:
        errors.append(f"Unclosed parentheses: {open_parens}")
    
    # Check for valid diagram types
    valid_types = ['graph TD', 'graph LR', 'sequenceDiagram', 'gantt', 'erDiagram', 'flowchart', 'stateDiagram']
    diagram_type = None
    for line in lines[:3]:  # Check first 3 lines
        for vtype in valid_types:
            if vtype in line:
                diagram_type = vtype
                break
        if diagram_type:
            break
    
    if not diagram_type:
        errors.append("No valid diagram type found")
    
    # Check for arrow syntax issues
    arrow_patterns = ['-->', '---', '==>', '==', '-.->', '-.-']
    has_arrows = any(pattern in diagram_content for pattern in arrow_patterns)
    
    if not has_arrows and diagram_type and 'graph' in diagram_type:
        errors.append("Graph diagram missing arrow connections")
    
    # Check for style syntax
    style_lines = [line for line in lines if line.strip().startswith('style ')]
    for style_line in style_lines:
        if not re.match(r'style\s+\w+\s+fill:', style_line):
            errors.append(f"Invalid style syntax: {style_line.strip()}")
    
    return errors, diagram_type

def test_mermaid_rendering(diagram_content):
    """Test if mermaid diagram can be rendered (requires mermaid CLI)"""
    try:
        # Create a temporary file
        import tempfile
        with tempfile.NamedTemporaryFile(mode='w', suffix='.mmd', delete=False) as f:
            f.write(diagram_content)
            temp_file = f.name
        
        # Try to render with mermaid CLI if available
        try:
            result = subprocess.run([
                'mmdc', '-i', temp_file, '-o', '/dev/null', '--quiet'
            ], capture_output=True, text=True, timeout=10)
            
            if result.returncode == 0:
                return True, "Rendered successfully"
            else:
                return False, f"Rendering failed: {result.stderr}"
        except (subprocess.TimeoutExpired, FileNotFoundError):
            return None, "Mermaid CLI not available"
        finally:
            os.unlink(temp_file)
    except Exception as e:
        return False, f"Rendering test error: {e}"

def main():
    """Main verification function"""
    print("🔍 MERMAID DIAGRAM VERIFICATION SCRIPT")
    print("=" * 50)
    
    # Find all markdown files in docs directory
    docs_dir = Path("docs")
    if not docs_dir.exists():
        print(f"❌ Docs directory not found: {docs_dir}")
        return
    
    markdown_files = list(docs_dir.glob("*.md"))
    
    if not markdown_files:
        print(f"❌ No markdown files found in {docs_dir}")
        return
    
    print(f"📁 Found {len(markdown_files)} markdown files")
    
    all_diagrams = []
    total_errors = 0
    
    # Extract all diagrams
    for file_path in markdown_files:
        diagrams = extract_mermaid_diagrams(file_path)
        all_diagrams.extend(diagrams)
        print(f"📄 {file_path.name}: {len(diagrams)} diagrams")
    
    print(f"\n🔍 Found {len(all_diagrams)} total mermaid diagrams")
    print("=" * 50)
    
    # Validate each diagram
    validation_results = []
    
    for diagram in all_diagrams:
        print(f"\n📊 Testing: {diagram['id']}")
        print(f"📍 File: {diagram['file']}")
        
        # Basic syntax validation
        errors, diagram_type = validate_mermaid_syntax(diagram['content'])
        
        result = {
            'id': diagram['id'],
            'file': diagram['file'],
            'type': diagram_type,
            'syntax_errors': errors,
            'render_test': None,
            'render_message': None
        }
        
        if errors:
            print(f"❌ Syntax Errors: {len(errors)}")
            for error in errors:
                print(f"   - {error}")
            total_errors += len(errors)
        else:
            print(f"✅ Syntax Valid")
        
        # Rendering test (if mermaid CLI available)
        render_result, render_message = test_mermaid_rendering(diagram['content'])
        result['render_test'] = render_result
        result['render_message'] = render_message
        
        if render_result is True:
            print(f"✅ Rendering: {render_message}")
        elif render_result is False:
            print(f"❌ Rendering: {render_message}")
            total_errors += 1
        else:
            print(f"⚠️ Rendering: {render_message}")
        
        validation_results.append(result)
    
    # Summary
    print("\n" + "=" * 50)
    print("📋 VALIDATION SUMMARY")
    print("=" * 50)
    
    valid_diagrams = len([r for r in validation_results if not r['syntax_errors']])
    print(f"✅ Valid Diagrams: {valid_diagrams}/{len(all_diagrams)}")
    print(f"❌ Total Errors: {total_errors}")
    
    if total_errors == 0:
        print("\n🎉 ALL MERMAID DIAGRAMS ARE VALID!")
    else:
        print(f"\n⚠️ Found {total_errors} errors in mermaid diagrams")
        print("Files with errors:")
        
        files_with_errors = set()
        for result in validation_results:
            if result['syntax_errors'] or result['render_test'] is False:
                files_with_errors.add(result['file'])
        
        for file_path in files_with_errors:
            print(f"  - {file_path}")
    
    # Save detailed results
    results_file = "mermaid_validation_results.json"
    with open(results_file, 'w') as f:
        json.dump(validation_results, f, indent=2)
    
    print(f"\n📄 Detailed results saved to: {results_file}")
    
    return total_errors == 0

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)