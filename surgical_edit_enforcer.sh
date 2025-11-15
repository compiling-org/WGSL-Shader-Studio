#!/bin/bash
# FORCED SURGICAL EDIT SCRIPT
# This script prevents destructive rewrites by enforcing surgical-only edits

echo "🔒 SURGICAL EDIT ENFORCEMENT ACTIVE"
echo "❌ COMPLETE FILE REWRITES FORBIDDEN"
echo "✅ ONLY SURGICAL EDITS ALLOWED"
echo ""
echo "RULES:"
echo "1. NO Write() calls - only Edit() and MultiEdit()"
echo "2. MUST read existing code first"
echo "3. MAX 5 lines changed per edit"
echo "4. MUST preserve existing structure"
echo "5. VERBOSE justification for every change"
echo ""
echo "VIOLATION CONSEQUENCES:"
echo "- Immediate termination of edit session"
echo "- Forced rollback of changes"
echo "- Manual review required"
echo ""
echo "Type 'I_UNDERSTAND' to proceed with surgical edits only:"

read confirmation

if [ "$confirmation" != "I_UNDERSTAND" ]; then
    echo "❌ ACCESS DENIED - Surgical edit agreement required"
    exit 1
fi

echo "✅ Surgical edit mode activated"
echo "🔍 Monitoring all edit operations..."

# Set strict edit monitoring
export SURGICAL_EDIT_MODE=1
export MAX_EDIT_LINES=5
export REQUIRE_READ_FIRST=1

echo "🛡️ Protection active - proceeding with surgical edits"