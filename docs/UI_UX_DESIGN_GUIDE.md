# UI/UX Design Guide

Principles and patterns for a professional shader studio UI.

## Visual Style
- Dark theme; high-contrast option; technical typography.
- Clear hierarchy; minimal decoration; consistent iconography.

## Layout & Panels
- Dockable/resizable panels; workspace presets.
- Main panels: Shader Browser, Code Editor, Live Preview, Parameters, Performance.

## Interaction Patterns
- Keyboard-first navigation; command palette; consistent shortcuts.
- Status overlays for long operations and initialization phases.
- Non-blocking errors: surfaces in editor and preview without panics.

## Accessibility
- Keyboard navigation; focus rings; labels for controls.
- Configurable font sizes and contrast.

## Acceptance Checklist (UI)
- Panels render after initialization without panics.
- Shortcuts documented and functional.
- Errors are visible, actionable, and do not crash the app.