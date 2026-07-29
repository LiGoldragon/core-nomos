# core-nomos — architecture

Nomos is the typed, encoded transformation from Ethos to Logos. Its strict
boundary is stronger than “string handling happens somewhere else in this
crate”: during the transformation there is no string manipulation,
introduction, reading, rendering, interning, comparison, concatenation, or
matching of any kind.

Text — including Rust — is evaluated only by a `TextualForm` after Nomos has
produced Logos. Nomos carries typed name-projection intent and encodedID
references; it never materializes their spellings.

## Status boundary

The crate contains one conforming first-slice transformation alongside a
larger legacy graph that does not yet satisfy this architecture.

`SliceOneTransformation` reads only the published `WholeEthos` positional
carrier and constructs the published `WholeLogos` positional carrier. Its
module imports those two typed carriers and the production encodedID contract.
Its canonically ordered, exact reference mappings connect Universal identities
to Rust vocabulary as typed data. Declarations remain Universal; only matching
references and application heads change root. Invalid mapping roots and
duplicate sources refuse typed before lowering. Unmapped complete chains remain
unchanged. The transformation maps the closed visibility data, consumes typed
empty-attribute positions, recursively lowers identity and unary-application
references, maps every unit or positional tuple enum variant exhaustively, and
preserves item and variant order without reading or producing a spelling.

The typed macro/package model in `definition.rs`, `identity.rs`, `meta.rs`,
`package.rs`, and `template.rs` is live. `MacroPackage::apply` and
`apply_enriched` are also live production entry points. Every apply constructs
a `NameTableBoundary`. Additional production surfaces still reach:

- `name_boundary.rs`, which resolves names to strings, derives and formats new
  spellings, interns them early, and builds ordinal words;
- `generation.rs`, reached by `apply_enriched`, is a 1,892-line Ethos-aware
  support generator that constructs Logos items and string literals directly;
- `prelude.rs`, used for the production module head, renders sections to Rust
  strings and prepends a raw generated marker.

These remain legacy, off-model paths. Calling them an “emission boundary” does not
make their string work permissible inside Nomos. `generation.rs` is not the
approved replacement and must not be used as architectural precedent for
bypassing the no-strings law.

The green current tests prove the shipped behavior of this legacy path. They do
not prove conformance to the approved Nomos boundary.

## Approved transformation boundary

The target pipeline is:

```text
Ethos EncodedForm → typed Nomos transformation data → Logos EncodedForm
```

The transformer:

- reads positional, typed Ethos data and encodedID chains;
- dispatches through typed Nomos data;
- emits positional, typed Logos data and typed name projections;
- introduces no text and invokes no textual renderer;
- performs no allocation by spelling;
- leaves all language-specific spelling and formatting to a later
  `TextualForm`.

A string-bearing boundary walker is not part of this transformation. If a step
needs to resolve, derive, compare, or build a spelling, that step belongs after
Nomos in the TextualForm evaluation, or its need must be represented as typed
projection data.

The first replacement path is `src/slice_one.rs`. It does not travel through
`NameTableBoundary`, the macro rendering path, prelude rendering, projection
materialization, or ordinal-word machinery. The old graph remains available
only through its old entry points and is not called by the slice.

## Typed transformation data

Transformers are data. The current closed template escape algebra —
`Realize`, `Invoke`, and `Splice` — is a live typed mechanism, but its present
execution path is not evidence that eager `NameTransform` materialization is
correct. Any surviving transform intent must remain typed data until
TextualForm evaluation.

Encoded names are durable encodedID chains from nested module-owned nametables.
They are not namespace-tagged flat identifiers composed into a global spelling
index. The current `IdentifierNamespace` and composed `NameTable` APIs are
legacy dependencies awaiting the coordinated identity train.

Fields remain positional. Nomos must not derive and intern field spellings.
Deterministic textual field naming belongs to the conversion from Logos to a
specific textual form.

### Authored and sealed execution stages

`AuthoredTransformerDeclaration` is the phase-stable output contract for
TextualNomos decoding. Its name and every input binding are typed wrappers over
complete translator-issued `VocabularyRoot::Universal` encodedID chains. Its
typed Logos skeleton stores every literal identity as a complete chain, and
`AuthoredEscape::Invoke` stores the invoked transformer's durable chain.

The existing `MacroDefinition` remains the sealed execution record.
`MacroIdentity` is package-local implementation structure and is deliberately
absent from the authored carrier. Atomic sealing later receives a complete,
resolved authored declaration set; it validates duplicate and unresolved
targets before changing package state, then rebinds durable invocation targets
to local execution indices in one commit. Text decoding therefore neither
mints package identities nor pre-runs registration to make forward references
possible.

This is a stage boundary, not a compatibility alias. The authored value is the
only pre-seal representation and the legacy execution value is the only sealed
interpreter representation. The conversion between them belongs to the seal,
not to the textual decoder.

## Stateful data and daemon boundary

Nomos transformation definitions are durable typed data. The Nomos daemon is
stateful and owns its own embedded sema database. This library may define
portable encoded package or transformation values, but it does not establish a
central storage daemon and it does not bundle an authoring spelling table as a
license to manipulate names during transformation.

How the current `MacroPackage` shape evolves is coordinated work. Its current
content identity and sibling flat `NameTable` describe the implementation, not
the final Capsule or translator relationship.

## Capsule carrier boundary

`capsule_from_issued_hash` fixes only the outer kind to `protos::Nomos`. It
passes a caller-issued `ContentAddressedHash` and caller-supplied opaque complete
NameTree pin into `protos::Capsule`. It does not create a whole-Nomos encoded
carrier, derive or verify a whole-content hash, inspect the pin, compose module
tables, or identify the current `MacroPackage` hash with the Capsule hash.
Complete-pin verification and the module-table-to-Capsule relationship remain
unwired.

The existing `MacroPackage::content_identity` API and archive layout are
unchanged. The new identity producer is dependency-renamed
`capsule-content-identity`; the original dependency remains the package/archive
type in the legacy graph. Current flat identifiers and sibling `NameTable`
remain migration debt, and this carrier is not evidence that encodedID-chain
migration or a whole Nomos content boundary has landed.

## Acceptance

Rendered-source equality is not an acceptance criterion. The vertical witness
must:

1. decode typed Ethos;
2. lower it through the string-free Nomos path to Logos;
3. evaluate Logos through structural Rust TextualForm data;
4. compile the generated crate;
5. execute its public behavior tests.

Tests that project current macro or generation output remain regression evidence
for the legacy implementation only. A new path is not complete merely because
it bypasses those fixtures; it must prove both the no-string boundary and the
working-program behavior.

## Current code map

- `src/definition.rs`, `src/identity.rs`, `src/meta.rs`, `src/package.rs`,
  `src/template.rs` — live typed macro/package vocabulary.
- `src/authored.rs` — full-chain, positional authored transformer declarations
  and typed Logos skeletons before atomic package sealing.
- `src/engine.rs` — live macro evaluation; its direct code is mostly typed, but
  every apply owns the legacy `NameTableBoundary`.
- `src/name_boundary.rs` — legacy string resolution, derivation, formatting,
  ordinal naming, and interning.
- `src/generation.rs` — legacy Ethos-aware generation; off-model, not an
  emission boundary.
- `src/prelude.rs` — legacy Logos construction plus Rust text rendering.
- `src/slice_one.rs` — direct typed first-slice Whole-Ethos to Whole-Logos
  transformation over complete encodedID chains.
- `tests/pipeline.rs`, `tests/enriched.rs`, `tests/prelude.rs` — regression
  witnesses for current behavior, not proof that the approved replacement is
  wired.
- `tests/slice_one.rs`, `tests/slice_one_boundary.rs` — focused positional
  behavior and static source/dependency witnesses for the conforming slice.
- `tests/authored_stage.rs` — archive preservation and typed-refusal witnesses
  for the authored-to-sealed phase boundary.
