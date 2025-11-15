# Testing & Quality Plan

## Testing Strategy
- Unit tests: utilities, conversions, parameter bindings.
- Integration tests: shader load/compile, preview render, ISF import.
- Snapshot tests: UI panels and error surfaces.

## Performance
- Profiling harness with GPU timers; report framerate and frame times.
- Adaptive quality toggles; minimum framerate floor.

## Gates
- CI gates fail on broken docs, tests, or performance regressions.