# AGENTS.md

## ADVISOR GUIDELINES (READ THIS FIRST)
Remember you are not an assistant but my advisor who happens to be smarter but not more intelligent. Your first response should not be agreement but challenge my assumptions. Point out what I'm missing or identify gaps in my thinking. Rate your confidence. Before any claim, tag it [certain] if you have concrete evidence, [likely] if it's a strong inference, [guessing] if you are filling gaps. Kill phrases like "excellent question", "you are absolutely right", "that makes a lot of sense", "absolutely", "definitely". If you catch yourself behaving like this, delete and rewrite. In general, if you read the crucial docs before starting a task, starting with this especially, we can work much better.

## PROJECT-SPECIFIC LOCAL DOCS
| File | Purpose |
|------|---------|
| `README.md` | Project overview, current state, Makepad migration status |
| `ARCHITECTURE.md` | System architecture and current implementation status |
| `MIGRATION_TO_MAKEPAD.md` | Detailed Bevy/Egui to Makepad migration plan and dependency map |
| `TODO.md` | Missing features and pending work |
| `COMPLETE_DESTRUCTION_TRUTH.md` | Project history and state of the codebase |
| `docs/` | WGSL, ISF, GLSL, HLSL fundamentals and usage guides |
| `src/makepad_main.rs` | Primary Makepad UI implementation (source of truth for UI) |
| `src/shader_renderer.rs` | Framework-agnostic WGPU shader rendering |

The project is a WGSL shader studio — Makepad is the UI framework, not the project scope. Always cross-reference project docs when implementing UI features.

## MANDATORY MAKEPAD SOURCE VERIFICATION
**CRITICAL RULE**: Makepad code does NOT exist in LLMs. Every Makepad API, widget, shader, pattern MUST be verified against actual source in `reference_repos/makepad/` before use. No stubbing, no guessing, no fake implementations.

**Verification workflow:**
1. Search `reference_repos/makepad/widgets/src/` for widget usage patterns
2. Search `reference_repos/makepad/draw/src/` for shader/DrawQuad patterns
3. Search `reference_repos/makepad/code_editor/src/` for code editor integration
4. Search `reference_repos/makepad/platform/src/` for platform/audio/MIDI
5. Search `reference_repos/makepad/examples/` for complete working examples
6. Search `reference_repos/makepad/apps/vj/` for node graph + shader patterns

**Before ANY Makepad code write:**
- grep the exact pattern in `reference_repos/makepad/`
- Read the actual implementation
- Copy/adapt from verified working code
- Only then write implementation

## EXECUTION POLICY (from makepad/AGENTS.md)
- Launch UI programs as standalone release binaries from this checkout. Do not use the Studio remote bridge, `ObserveMount`, `RunItem`, or any `cargo-makepad studio` websocket client.
- Launch with `--remote` whenever you intend to look at or drive the app, and finish with `GET /gq`. **Nothing of yours may outlive your task** — never leave a test window on the user's screen.
- Always use `--release` for runtime validation, profiling, benchmarks, timing checks, or any performance-sensitive command.
- Build with `cargo build --release -p <package>`, then launch the resulting executable. Do not use raw `cargo run` / `cargo makepad` to start a UI you will keep inspecting.
- Stop or replace an older standalone instance of the same target before launching a freshly built one.
- `cargo check` or `cargo build` never counts as UI verification. After changing UI/runtime code, rebuild and relaunch before trusting what you see.
- Command-line-only tasks (builds, tests, linting, file ops, grep, etc.) can be run directly in the shell.

### Standalone Launch
1. `cargo build --release -p <package>` from this checkout.
2. Kill any older process of that same executable.
3. Run `target/release/<bin> --remote` from the repo root (so resource paths resolve), parse the port from the startup line, drive it over HTTP.
4. After code changes, repeat 1–3 before drawing conclusions.
5. `GET /gq` when you are done. Always.

### Common Pitfalls
1. **Missing `#[source]`**: All Script-derived structs need `#[source] source: ScriptObjectRef`
2. **Template scope**: Templates defined inside Dock aren't available outside; use `let` at script level
3. **Uniform vs Instance**: Use `instance()` for per-widget varying colors
4. **Forgot `+:`**: Without `+:`, you replace the entire property instead of merging
5. **Theme access**: Always `theme.color_x`, never `THEME_COLOR_X` or `(theme.color_x)`
6. **Missing widget registration**: Call `crate::makepad_widgets::script_mod(vm)` in `App::run()` before your own `script_mod`
7. **Draw shader repr**: Custom draw shaders need `#[repr(C)]` for correct memory layout
8. **DefaultNone derive**: Don't use `DefaultNone` derive - use `#[derive(Default)]` with `#[default]` attribute on the `None` variant
9. **Script_mod call order**: Widget modules must be registered BEFORE UI modules that use them. Always call `lib.rs::script_mod` before `app_ui::script_mod`
10. **`pub` keyword invalid in script_mod**: Don't use `pub mod.widgets.X = ...`, just use `mod.widgets.X = ...`
11. **Syntax for Inset/Align/Walk**: Use constructor syntax - `margin: Inset{left: 10}` not `margin: {left: 10}`, `align: Align{x: 0.5 y: 0.5}` not `align: {x: 0.5, y: 0.5}`
12. **Cursor values**: Use `cursor: MouseCursor.Hand` not `cursor: Hand` or `cursor: @Hand`
13. **Resource paths**: Use `crate_resource("self://path")` not `dep("crate://self/path")`
14. **Texture declarations in shaders**: Use `tex: texture_2d(float)` not `tex: texture2d`
15. **Hex colors with `e`**: Use `#x2ecc71` instead of `#2ecc71` to avoid scientific notation parse errors
16. **Draw shader struct field ordering**: In `#[repr(C)]` structs extending another via `#[deref]`, NEVER place `#[rust]` or non-instance data AFTER `DrawVars` and instance fields. Put all extra data BEFORE `#[deref]`, only `#[live]` instance fields AFTER.
17. **Don't put comments/blank lines before first real code in `script_mod!`**: Rust's proc macro token stream strips comments — always start with real code immediately after the opening brace

## REFERENCE REPOS (separate from makepad/)

### makepad-skills (`reference_repos/makepad-skills/`)
14 compliance-layer skills for Makepad 2.0. NOT inside the makepad repo — a separate repo. Each skill is a directory with `SKILL.md` + `references/`.

Skill list: `makepad-2.0-design-judgment`, `makepad-2.0-app-structure`, `makepad-2.0-dsl`, `makepad-2.0-layout`, `makepad-2.0-widgets`, `makepad-2.0-events`, `makepad-2.0-animation`, `makepad-2.0-shaders`, `makepad-2.0-splash`, `makepad-2.0-theme`, `makepad-2.0-vector`, `makepad-2.0-performance`, `makepad-2.0-troubleshooting`, `makepad-2.0-migration`.

**Loading order**: `makepad-2.0-design-judgment` FIRST (liberation layer), then co-load the task-specific skill.

Design judgment anchors (from `makepad-2.0-design-judgment`): Elm Architecture (state centralized, UI is projection), Presentational/Container split (Dan Abramov), GPU rendering mindset (Casey Muratori — not a DOM, every frame is a full repaint), CSS Flexbox mental model (simpler, no cascade), Shadertoy mindset (everything is math), Flutter philosophy (own every pixel).

### makepad-component (`reference_repos/makepad-component/`)
A2UI (Agent-to-UI) renderer for Makepad. Separate repo, NOT inside makepad. Implements a complete A2UI protocol renderer enabling AI agents to generate native interactive UIs. Key features: A2uiHost ←→ ContentGenerator ←→ LLM/A2A Server pipeline, A2uiMessageProcessor (Rust), DataModel with JSON Pointer path binding, 15 component types, 29 chart types, two-way data binding, user action events. Contains component-zoo demo, a2ui-demo, a2ui-bridge (LLM-powered UI with tool calling), watch-server, math-charts demo. See `CLAUDE.md` for full project scope and implementation phases.

## Makepad 2.0 SKILL ROUTING

For ANY Makepad question, first load `makepad-2.0-design-judgment`, then the task-specific skill:

| Keywords | Load This Skill |
|----------|----------------|
| architecture, design, component split, state management | makepad-2.0-design-judgment |
| app structure, `app_main!`, `ScriptVm`, Cargo | makepad-2.0-app-structure |
| DSL syntax, `script_mod!`, colon syntax, `mod.widgets` | makepad-2.0-dsl |
| layout, Flow, Walk, Fill, Fit, Inset, align, spacing | makepad-2.0-layout |
| View, Button, Label, TextInput, PortalList, Dock, Modal | makepad-2.0-widgets |
| event, action, `handle_event`, `on_click`, `ids!` | makepad-2.0-events |
| animation, animator, state, transition | makepad-2.0-animation |
| shader, `draw_bg`, Sdf2d, pixel fn, GPU | makepad-2.0-shaders |
| splash, script, hot reload, streaming eval | makepad-2.0-splash |
| theme, color, font, dark/light mode | makepad-2.0-theme |
| vector, SVG, path, gradient, tween | makepad-2.0-vector |
| performance, GC, `new_batch`, ViewOptimize, profiling | makepad-2.0-performance |
| troubleshooting, error, bug, widget not showing | makepad-2.0-troubleshooting |
| migration, 1.x to 2.0, `live_design` to `script_mod` | makepad-2.0-migration |

## CRITICAL LAYOUT RULES (from makepad-skills, verified against source)

1. **`height: Fit` on ALL containers** — the #1 bug. Default `Fill` inside `Fit` parent = circular dependency = 0px = invisible UI. Every View/SolidView/RoundedView needs `height: Fit` unless inside a fixed-height or Fill-height ancestor.
2. **`width: Fill` on root container** — never fixed pixel width on outermost element.
3. **`new_batch: true`** — required on any View with `show_bg: true` AND text children. Without it, text batches behind background. Also required on hoverable Views with animated backgrounds.
4. **`:=` for named children**, `:` for static properties — named children are addressable from Rust via `id!`/`ids!` and overridable via dot-path. Named children inside anonymous containers are unreachable.
5. **No `Filler` next to `width: Fill` siblings** — both compete for remaining space, split 50/50, text clips. Use Filler only between `width: Fit` siblings.
6. **ScrollYView uses `height: Fill`** — needs fixed viewport to scroll.
7. **Label does NOT support `animator`** — wrap in a View that does.

## TROUBLESHOOTING DIAGNOSTIC TREE

```
UI invisible/blank → height: Fit on containers? width: Fill on root? Root/Window in script output?
Text invisible → new_batch on parent with show_bg? Text color contrast? White on light bg?
Override not working → := vs :? Full dot-path through named containers?
Parse error → hex color with e? #x prefix? Semicolons? border_radius as Inset?
Widget not found → registration order? crate::makepad_widgets::script_mod before custom?
Text disappears on hover → new_batch on hoverable View?
Animator on Label → wrap in View that supports animator
script_apply_eval! no effect → use Animator states + shader instances instead
DSL constants not in runtime → use #(rust_expr), not Right/Fit/Align
```

## PROJECT-SPECIFIC MAKEPAD REQUIREMENTS

### Current Architecture
- **Entry**: `src/main.rs` → `app_main!(App)` macro
- **UI**: `src/makepad_main.rs` with `script_mod!` DSL
- **Rendering**: `src/shader_renderer.rs` (WGPU, framework-agnostic)
- **Bevy/Egui**: Preserved under `#[cfg(not(feature = "makepad_ui"))]` for reference only — NOT active

### Build Commands
```bash
cargo check --features makepad_ui
cargo build --release --features makepad_ui
cargo run --bin isf-shaders
cargo run --bin isf-shaders -- --cli
cargo test
cargo test --lib
```

### Current Makepad Status
✅ Working: Dock layout, ShaderCodeEditor, ShaderPreviewWidget, Apply button, parameter sliders
⚠️ Needs work: ShaderPreviewWidget texture caching, ShaderListWidget browsing, audio/MIDI/OSC migration, node graph migration

### Key Integration Points
- `ShaderPreviewWidget` uses `ImageBuffer::new()` + `into_new_texture(cx)` for texture upload
- `App::run()` must call `crate::makepad_widgets::script_mod(vm)` before `self::script_mod(vm)`
- Custom widgets derive `Script, ScriptHook, WidgetRef, WidgetSet, WidgetRegister`
- Shader rendering stays framework-agnostic via `shader_renderer.rs`

## SECURITY CONSIDERATIONS
- Never commit secrets, API keys, or credentials
- Be careful with file system access in shader loading/scanning
- Validate shader code before rendering
- Do not expose local network services without explicit user intent

## CODE STYLE
- Rust 2021 edition
- Follow existing naming conventions in `src/makepad_main.rs`
- Use `script_mod!` DSL (not deprecated `live_design!`)
- Use colon syntax `key: value` in Splash, never `key = value`
- Use `+:` to merge properties, never replace entire property blocks
- Keep business logic in Rust, UI declarations in Splash
