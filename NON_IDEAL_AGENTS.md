# Non-idealities — core-nomos

Recorded debt with a future fix target. Ordinary rules live in `AGENTS.md`; the
ideal shape lives in `ARCHITECTURE.md`. Each entry names the symptom, the current
workaround, and the proper fix or the design question the psyche must settle.

There are currently no open non-idealities.

## Resolved

### The class-D `TraceEvent` tuple-struct declaration is not projectable

Resolved by the layout-4 tuple-field-visibility kernel slice. `core-logos`
`Newtype` gained a stored `wrapped_visibility: Visibility` (layout 3 → 4), so the
public tuple-field form is modeled and `textual-rust` can read and project it. The
class-D `TraceSupport` generator emits the `TraceEvent` declaration in document order
between the `ObjectName` enum and `impl ObjectName`. Structural coverage now lives in
the enriched generator tests; program behavior is covered by the process witness.
