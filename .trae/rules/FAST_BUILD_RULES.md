# FAST BUILD COMPILATION RULES

## MANDATORY BUILD OPTIMIZATION PROTOCOL

### CRITICAL REQUIREMENTS
- **ALWAYS USE `--release` for production builds** - 10x faster execution
- **ALWAYS USE `--release` for testing UI functionality** - prevents lag and stuttering
- **Use dev profile for rapid iteration** when fixing compilation errors only

### PROHIBITED PRACTICES
- ❌ NEVER run debug builds for UI testing (causes severe performance issues)
- ❌ NEVER use plain `cargo build` for final verification
- ❌ NEVER test shader compilation or rendering in debug mode

### MANDATORY COMMANDS
```bash
# For ALL UI testing and final verification:
cargo run --release --bin [target]

# For rapid compilation error fixing:
cargo build --release

# For development iteration (compilation errors only):
cargo build  # Fast compilation check
cargo run --bin [target]  # Only if testing compilation, not UI
```

### PERFORMANCE IMPACT
- Debug builds: 5-10x slower shader compilation
- Debug builds: 2-3x slower UI rendering  
- Debug builds: Frame drops and stuttering in preview panels
- Release builds: Optimal performance for all shader operations

### VIOLATION CONSEQUENCES
- UI panels will appear broken due to performance issues
- Shader compilation timeouts in debug mode
- False positive error reports from slow execution
- Disciplinary protocol activation for repeated violations

### AUTOMATIC ENFORCEMENT
This rule is automatically monitored by the session enforcer script. Any deviation from release builds for UI functionality will trigger violation logging.