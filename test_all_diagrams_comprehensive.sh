#!/bin/bash

echo "🔍 COMPREHENSIVE MERMAID DIAGRAM VALIDATION"
echo "=========================================="
echo ""

# Function to test a mermaid diagram
test_diagram() {
    local name="$1"
    local diagram="$2"
    local file="$3"
    
    echo "Testing: $name (from $file)"
    echo "Diagram content:"
    echo "$diagram"
    echo ""
    
    # Create temporary file for testing
    echo "$diagram" > temp_diagram.mmd
    
    # Test with mmdc if available, otherwise basic syntax check
    if command -v mmdc >/dev/null 2>&1; then
        if mmdc -i temp_diagram.mmd -o temp_output.png >/dev/null 2>&1; then
            echo "✅ VALID - Successfully rendered"
        else
            echo "❌ INVALID - Rendering failed"
        fi
    else
        # Basic syntax validation
        if echo "$diagram" | grep -q "graph\|sequenceDiagram\|gantt\|classDiagram\|stateDiagram"; then
            echo "✅ SYNTAX VALID - Basic structure detected"
        else
            echo "❌ SYNTAX INVALID - No valid diagram type found"
        fi
    fi
    
    rm -f temp_diagram.mmd temp_output.png
    echo "----------------------------------------"
    echo ""
}

# Test Technology Stack diagrams
echo "TESTING TECHNOLOGY_STACK.md DIAGRAMS"
echo "====================================="

# Diagram 1: Technology Stack
tech_stack1='graph TD
    A[Technology Stack] --> B[Bevy 0.17 + bevy_egui 0.38]
    A --> C[❌ eframe FORBIDDEN]
    A --> D[❌ eframe::egui FORBIDDEN]
    
    B --> E[✅ Main Entry: src/bevy_app.rs::run_app()]
    B --> F[✅ UI Context: bevy_egui::EguiContexts]
    B --> G[✅ Window Management: Bevy WindowPlugin]
    
    C --> H[⚠️ COMPLETE APPLICATION BREAK]
    D --> H
    
    style A fill:#ffebee
    style B fill:#e8f5e9
    style C fill:#ffcdd2
    style D fill:#ffcdd2
    style H fill:#f44336'

test_diagram "Technology Stack Overview" "$tech_stack1" "TECHNOLOGY_STACK.md"

# Diagram 2: Import Verification
import_verify='graph LR
    A[Import Verification] --> B{Check Imports}
    B -->|✅ bevy_egui| C[ALLOWED]
    B -->|❌ eframe| D[FORBIDDEN]
    
    C --> E[Continue Development]
    D --> F[INSTANT REVERT REQUIRED]
    
    style A fill:#e3f2fd
    style C fill:#4caf50
    style D fill:#f44336
    style F fill:#ffebee'

test_diagram "Import Verification" "$import_verify" "TECHNOLOGY_STACK.md"

# Diagram 3: Application Entry
app_entry='graph TD
    A[Application Entry] --> B{Choose Path}
    B -->|✅ bevy_app::run_app()| C[Bevy + bevy_egui]
    B -->|❌ gui::run_gui()| D[eframe - FORBIDDEN]
    
    C --> E[Working Application]
    D --> F[Complete Failure]
    
    style A fill:#fff3e0
    style C fill:#e8f5e9
    style D fill:#ffcdd2
    style F fill:#f44336'

test_diagram "Application Entry Flow" "$app_entry" "TECHNOLOGY_STACK.md"

# Diagram 4: Bevy App Architecture
bevy_arch='graph TD
    A[Bevy App] --> B[DefaultPlugins]
    A --> C[EguiPlugin]
    A --> D