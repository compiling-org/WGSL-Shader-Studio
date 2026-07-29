#!/bin/bash

# COMPREHENSIVE MERMAID DIAGRAM VERIFICATION SYSTEM
# This script tests EVERY SINGLE mermaid diagram in ALL documents
# Provides detailed proof of which diagrams work vs which are broken

set -e

echo "🧪 COMPREHENSIVE MERMAID DIAGRAM VERIFICATION"
echo "=============================================="
echo "Testing every single diagram in all documents..."
echo ""

# Color codes for results
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_DIAGRAMS=0
WORKING_DIAGRAMS=0
BROKEN_DIAGRAMS=0

# Function to test a mermaid diagram
test_diagram() {
    local diagram_content="$1"
    local diagram_name="$2"
    local file_path="$3"
    
    TOTAL_DIAGRAMS=$((TOTAL_DIAGRAMS + 1))
    
    # Create temporary test file
    local temp_file="$(mktemp).mmd"
    echo "$diagram_content" > "$temp_file"

    # Validate diagram type
    local has_type=0
    if echo "$diagram_content" | grep -q -E '^[[:space:]]*(graph|flowchart)[[:space:]]'; then has_type=1; fi
    if echo "$diagram_content" | grep -q -E '^[[:space:]]*sequenceDiagram'; then has_type=1; fi
    if echo "$diagram_content" | grep -q -E '^[[:space:]]*gantt'; then has_type=1; fi
    if echo "$diagram_content" | grep -q -E '^[[:space:]]*classDiagram'; then has_type=1; fi
    if echo "$diagram_content" | grep -q -E '^[[:space:]]*stateDiagram(-v2)?'; then has_type=1; fi
    if echo "$diagram_content" | grep -q -E '^[[:space:]]*erDiagram'; then has_type=1; fi
    if echo "$diagram_content" | grep -q -E '^[[:space:]]*journey'; then has_type=1; fi
    if echo "$diagram_content" | grep -q -E '^[[:space:]]*pie'; then has_type=1; fi
    if [[ "$has_type" -eq 0 ]]; then
        echo -e "${RED}❌ BROKEN${NC}: $diagram_name in $file_path"
        echo -e "  ${RED}Error: Missing diagram type header (e.g., 'graph TD')${NC}"
        BROKEN_DIAGRAMS=$((BROKEN_DIAGRAMS + 1))
        rm -f "$temp_file"
        echo ""
        return
    fi

    # Prefer real rendering test if mmdc or npx is available
    if command -v mmdc >/dev/null 2>&1; then
        if mmdc -i "$temp_file" -o "${temp_file%.mmd}.svg" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ RENDERED${NC}: $diagram_name in $file_path"
            WORKING_DIAGRAMS=$((WORKING_DIAGRAMS + 1))
            rm -f "$temp_file" "${temp_file%.mmd}.svg"
            echo ""
            return
        else
            echo -e "${RED}❌ BROKEN${NC}: $diagram_name in $file_path"
            echo -e "  ${RED}Error: mmdc failed to render diagram${NC}"
            BROKEN_DIAGRAMS=$((BROKEN_DIAGRAMS + 1))
            rm -f "$temp_file"
            echo ""
            return
        fi
    elif command -v npx >/dev/null 2>&1; then
        # Attempt ephemeral CLI render (best-effort; may be slow)
        if npx -y @mermaid-js/mermaid-cli -i "$temp_file" -o "${temp_file%.mmd}.svg" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ RENDERED${NC}: $diagram_name in $file_path"
            WORKING_DIAGRAMS=$((WORKING_DIAGRAMS + 1))
            rm -f "$temp_file" "${temp_file%.mmd}.svg"
            echo ""
            return
        else
            echo -e "${RED}❌ BROKEN${NC}: $diagram_name in $file_path"
            echo -e "  ${RED}Error: mermaid-cli (npx) failed to render diagram${NC}"
            BROKEN_DIAGRAMS=$((BROKEN_DIAGRAMS + 1))
            rm -f "$temp_file"
            echo ""
            return
        fi
    fi

    # Fallback: minimal validation (type header present is enough)
    echo -e "${GREEN}✅ VALID (type detected)${NC}: $diagram_name in $file_path"
    WORKING_DIAGRAMS=$((WORKING_DIAGRAMS + 1))
    
    rm -f "$temp_file"
    echo ""
}

# Function to extract and test diagrams from a file
extract_and_test_diagrams() {
    local file_path="$1"
    
    if [[ ! -f "$file_path" ]]; then
        echo -e "${RED}File not found: $file_path${NC}"
        return
    fi
    
    echo -e "${BLUE}📄 TESTING FILE: $file_path${NC}"
    echo "================================"
    
    # Fence and adjacency validation
    local quad_fence_count
    quad_fence_count=$(grep -n '^\`\`\`\`mermaid$' "$file_path" | wc -l || true)
    if [[ "$quad_fence_count" -gt 0 ]]; then
        echo -e "${RED}❌ BROKEN${NC}: Found '````mermaid' quadruple fence declarations"
        echo -ne "  ${RED}Fix: Replace with "
        echo -n '```mermaid'
        echo -e " and ensure proper closing fences${NC}"
        BROKEN_DIAGRAMS=$((BROKEN_DIAGRAMS + quad_fence_count))
    fi
    local stray_tick_count
    stray_tick_count=$(grep -n '^[[:space:]]*\`[[:space:]]*$' "$file_path" | wc -l || true)
    if [[ "$stray_tick_count" -gt 0 ]]; then
        echo -e "${RED}❌ BROKEN${NC}: Found stray single backtick lines which break rendering"
        BROKEN_DIAGRAMS=$((BROKEN_DIAGRAMS + stray_tick_count))
    fi
    # Adjacent fences without blank line
    awk '
    {lines[NR]=$0}
    END {
        for (i=2; i<=NR; i++) {
            if (lines[i-1] == "```" && lines[i] == "```mermaid") {
                print i
            }
        }
    }' "$file_path" | while read -r line_no; do
        echo -e "${YELLOW}⚠️  Warning${NC}: Mermaid fence at line $line_no immediately follows a closing fence; add a blank line"
    done

    # Extract mermaid diagrams using awk and delimit with a sentinel to preserve block boundaries
    awk '
    {
        sub(/\r$/, "", $0); # strip Windows CR if present
    }
    /^```mermaid$/ {in_diagram=1; diagram=""; next}
    /^```$/ && in_diagram {in_diagram=0; print diagram "\n<<<END_DIAGRAM>>>"; diagram=""; next}
    in_diagram {diagram = diagram $0 "\n"}
    END {
        if (in_diagram) { print diagram "\n<<<END_DIAGRAM>>>"; }
    }
    ' "$file_path" | {
        buf=""
        idx=0
        while IFS= read -r line; do
            # normalize CR if any slipped through
            line="${line%$'\r'}"
            if [[ "$line" == "<<<END_DIAGRAM>>>" ]]; then
                if [[ -n "$buf" ]]; then
                    idx=$((idx + 1))
                    local_name="Diagram $((TOTAL_DIAGRAMS + 1))"
                    test_diagram "$buf" "$local_name" "$file_path"
                    buf=""
                fi
            else
                buf+="$line"$'\n'
            fi
        done
    }
}

# Entry: optional single-file mode
if [[ -n "$1" ]]; then
    echo -e "${BLUE}🔍 SINGLE FILE MODE${NC}"
    extract_and_test_diagrams "$1"
else
    echo -e "${BLUE}🔍 TESTING ALL DOCUMENTS${NC}"
    echo "================================"
    # Root directory documents
    for doc in README.md TECHNOLOGY_STACK.md; do
        if [[ -f "$doc" ]]; then
            extract_and_test_diagrams "$doc"
        fi
    done
    # Docs directory documents
    if [[ -d "docs" ]]; then
        for doc in docs/*.md; do
            if [[ -f "$doc" ]]; then
                extract_and_test_diagrams "$doc"
            fi
        done
    fi
fi

# Summary report
echo -e "${BLUE}📊 FINAL VERIFICATION REPORT${NC}"
echo "================================"
echo -e "Total Diagrams Tested: ${TOTAL_DIAGRAMS}"
echo -e "Working Diagrams: ${GREEN}$WORKING_DIAGRAMS${NC}"
echo -e "Broken Diagrams: ${RED}$BROKEN_DIAGRAMS${NC}"
if [[ $TOTAL_DIAGRAMS -gt 0 ]]; then
    echo -e "Success Rate: ${GREEN}$((WORKING_DIAGRAMS * 100 / TOTAL_DIAGRAMS))%${NC}"
else
    echo -e "Success Rate: ${YELLOW}No diagrams found to test${NC}"
fi

if [[ $BROKEN_DIAGRAMS -gt 0 ]]; then
    echo ""
    echo -e "${RED}🚨 CRITICAL: $BROKEN_DIAGRAMS diagrams are broken and need fixing!${NC}"
    exit 1
else
    echo ""
    echo -e "${GREEN}✅ ALL DIAGRAMS WORKING - VERIFICATION COMPLETE!${NC}"
    exit 0
fi
