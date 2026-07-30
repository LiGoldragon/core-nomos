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

The crate contains the native authored evaluator and a smaller direct
first-slice reference transformation alongside a retained legacy graph. The
native evaluator is the production-bound door; engine integration source gates
must prevent the legacy graph from becoming reachable through deployment.

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

The canonical Ethos and Logos dependencies now carry both their full-chain
whole carriers and the retained flat execution data. There are no slice aliases
or parallel structural-codec type universes. The former structural-codec 0.6
`EncodedConversion` implementation over the flat records was removed rather
than adapted: the legacy engine retains its inherent `apply` methods, while
0.8 conversion contracts are reserved for canonical `EncodedForm` carriers.

The typed legacy macro/package model in `definition.rs`, `identity.rs`,
`meta.rs`, `package.rs`, and `template.rs` remains as a regression oracle.
`MacroPackage::apply` and `apply_enriched` still exist as public legacy entry
points, and every apply constructs a `NameTableBoundary`. The native evaluator
does not import or call them. The retained legacy graph reaches:

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

The legacy tests prove only that retained oracle. Native acceptance is carried
by the direct evaluator, archive, source, and process witnesses.

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

The production replacement path is `src/native.rs`.
`NativeAuthoredEvaluator::try_new` admits one immutable
`AuthoredTransformerSet`, rejecting text scalars, invalid identities,
unsupported invocation inputs, and invocation cycles before input execution.
`transform` consumes `EncodedPopulation<WholeEthos, NameTree>` and returns a
checked `NativeLogosPopulation` plus the current bounded `WholeLogos`
compatibility projection. Invoke and Splice substitute their actual typed
landing values; no operational future marker survives.

The NameTree boundary authenticates and plans before the evaluator visits the
first item. Derived-name realization and the complete output tree are pure plan
operations; the daemon commits planned names and output storage atomically only
after successful evaluation. This two-phase boundary is
`[to-be-reviewed-by-psyche]`.

`NativeReferenceUniverse` maps exact complete Universal reference identities
and application heads to exact Rust identities. It never compares local leaves;
same-leaf identities under different ancestors remain distinct. Unlisted
complete identities remain unchanged. This current Universal-to-Rust universe
law is `[to-be-reviewed-by-psyche]`.

Authored visibility literals are authoritative. Input visibility is absent from
the authored signature and therefore cannot override those literals. This
intentionally diverges from the po2.5 SliceOne/legacy visibility behavior and
is `[to-be-reviewed-by-psyche]`.

`src/slice_one.rs` remains the direct typed reference/bootstrap oracle. Neither
path travels through `NameTableBoundary`, the macro rendering path, prelude
rendering, projection materialization, or ordinal-word machinery.

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

`AuthoredTransformerSet` is itself the immutable sealed execution content.
Every declaration retains a typed `TemplateRootOutputSelector` identifying
either the complete result or one exact transparent landing role. Invoke uses
that sealed selector; it never infers an output from field count or fixture
position. Preserving this selector in the content preimage is
`[to-be-reviewed-by-psyche]`.

There is no production authored-to-`MacroPackage` conversion. `MacroIdentity`
and `PackageRevision` remain legacy oracle structure only.

## Stateful data and daemon boundary

Nomos transformation definitions are durable typed data. The Nomos daemon is
stateful and owns its own embedded sema database. This library may define
portable encoded package or transformation values, but it does not establish a
central storage daemon and it does not bundle an authoring spelling table as a
license to manipulate names during transformation.

How the current `MacroPackage` shape evolves is coordinated work. Its current
content identity and sibling flat `NameTable` describe the implementation, not
the final Capsule or translator relationship.

The temporary six-value `GenerationClass` selection is not evaluated or sealed
by the native library. The daemon carries it as separately typed per-slot
deployment metadata `[to-be-reviewed-by-psyche]`. po2.8 owns authoring that
behavior in Nomos and retiring the external selection.

## Capsule carrier boundary

`capsule_from_issued_hash` fixes only the outer kind to `protos::Nomos`. It
passes a caller-issued `ContentAddressedHash` and caller-supplied opaque complete
NameTree pin into `protos::Capsule`. It remains a legacy caller-issued
pass-through and does not identify the current `MacroPackage` hash with a
production Capsule hash.

The existing `MacroPackage::content_identity` API and archive layout are
unchanged as the po2.5 fixture oracle. Production `SealedNomosCapsule` identity
hashes only canonical `AuthoredTransformerSet` bytes through the
dependency-renamed `capsule-content-identity`. Versioned reachable NameTree
spellings live in the separate integrity-authenticated
`AuthenticatedNameTreeProjection`. No flat identity, package revision, spelling,
projection, provenance, cache, or live-slot state enters the content preimage.

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
  and typed Logos skeletons with sealed root-output selectors.
- `src/native.rs` — native authored evaluation, paired population/NameTree
  planning, exact reference universe, checked native Logos archive boundary,
  and bounded WholeLogos projection.
- `src/sealed.rs` — immutable content Capsule identity and the separately
  authenticated, versioned reachable NameTree projection.
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
- `tests/native.rs`, native unit tests, and the native witnesses in
  `tests/textual_nomos.rs` — production-door source exclusion, package
  admission, future substitution, exact reference mapping, NameTree/archive
  validation, and authored-literal authority.
- `tests/authored_stage.rs` — archive preservation and typed-refusal witnesses
  for the authored-to-sealed phase boundary.
- `tests/textual_nomos_authority.rs`, `tests/textual_nomos_manifest.rs` —
  receipt-backed seal, rename stability, projection reversibility, and
  canonical-order witnesses.
