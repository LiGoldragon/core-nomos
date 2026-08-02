# Non-idealities — core-nomos

Recorded debt with a future fix target. Ordinary rules live in `AGENTS.md`; the
ideal shape lives in `ARCHITECTURE.md`. Each entry names the symptom, the current
workaround, and the proper fix or the design question the psyche must settle.

## Handwritten Nomos/Logos mirror types until self-hosting

- **Every new Nomos object currently gets a corresponding handwritten Rust
  type that closely mirrors its Logos type.** The Nomos-side fields retain
  unresolved values from the Ethos payload while the Logos-side fields hold
  final concrete values. Maintaining these near-duplicate pairs by hand is a
  sanctioned bootstrap hack (psyche-ruled 2026-08-01,
  `design/ProtosEngine/traitStandardAndSpiritRename-2026-08-01.md`). Keep each
  pair structurally in step; drift is a defect, not a design choice.
- **Proper fix:** the self-hosting loop described in
  `design/ProtosEngine/threeLayerNamingAndNomosBootstrap-2026-08-01.md`,
  section 8. A specialized Nomos object consumes Ethos type declarations and
  emits both the final Logos type and its unresolved Nomos mirror. The
  handwritten pairs become the generator's first fixtures and are retired.

## Resolved

### The class-D `TraceEvent` tuple-struct declaration is not projectable

Resolved by the layout-4 tuple-field-visibility kernel slice. `core-logos`
`Newtype` gained a stored `wrapped_visibility: Visibility` (layout 3 → 4), so the
public tuple-field form is modeled and `textual-rust` can read and project it. The
class-D `TraceSupport` generator emits the `TraceEvent` declaration in document order
between the `ObjectName` enum and `impl ObjectName`. Structural coverage now lives in
the enriched generator tests; program behavior is covered by the process witness.
