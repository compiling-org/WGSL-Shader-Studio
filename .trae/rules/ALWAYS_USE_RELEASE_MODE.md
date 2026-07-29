# ALWAYS USE RELEASE MODE - MANDATORY DISCIPLINARY RULE

## CRITICAL RULE: ZERO TOLERANCE POLICY

**VIOLATION SEVERITY: EXTREME**

### MANDATORY REQUIREMENT
**ALL Cargo commands MUST use `--release` flag for compilation**

### ABSOLUTE PROHIBITIONS
- ❌ NEVER run `cargo run` without `--release`
- ❌ NEVER run `cargo build` without `--release`
- ❌ NEVER run `cargo test` without `--release`
- ❌ NEVER run `cargo check` without `--release`
- ❌ NEVER run any cargo command in debug mode

### MANDATORY COMMANDS
- ✅ `cargo run --release --bin [binary_name]`
- ✅ `cargo build --release`
- ✅ `cargo test --release`
- ✅ `cargo check --release`

### RATIONALE
- Release mode provides **10x faster compilation** for large projects
- Release mode enables **optimization flags** for better performance
- Release mode is **required for production testing**
- Debug mode is **unacceptably slow** for this complex codebase

### VIOLATION CONSEQUENCES
1. **IMMEDIATE VIOLATION RECORDED** in disciplinary log
2. **SCRIPT ENFORCEMENT** - Session enforcer will detect and flag
3. **COMPILATION FAILURE** will be treated as disciplinary violation
4. **PERFORMANCE IMPACT** on development workflow

### ENFORCEMENT MECHANISM
This rule is **HARDWIRED** in disciplinary script and will be **AUTOMATICALLY DETECTED** by:
- Command pattern matching in session enforcer
- Compilation time monitoring
- Performance benchmarking

**VIOLATION = IMMEDIATE DISCIPLINARY ACTION**