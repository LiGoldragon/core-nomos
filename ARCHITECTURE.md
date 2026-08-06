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

`BootstrapSliceOneLowering` is the current authority-sealed bootstrap boundary. It accepts
the same `BootstrapReader<A>` that sealed an authority-branded
`PreparedBootstrapTransaction<A>`, revalidates the authority receipt and every
prepared-model invariant, and then reads the transaction directly. It never
accepts a draft or decoded document and never reconstructs `WholeEthos`.
Canonically ordered Nexus declarations and role-free Interface support types
become ordered `WholeLogos` items. References and recursive Shape applications
retain complete identities; unit, unary, and arbitrary nonempty product variants
retain exact variant and field order. Trait declarations, Interface roles,
Streams, Sema tables, and Trait requirements each have a distinct typed refusal
that retains the unsupported identity or requirement.

The separate `lower_sema` operation is the storage-aware boundary; the generic
`lower` operation continues to refuse Sema. `lower_sema` marks the exact
document's record declarations as stored, requires every table record and key
to be owned by that document, requires each key to be a newtype, and emits a
first-class `WholeLogosTable`. Recursive local storage fingerprints use the
same domain-separated structural algorithm as full Nexus lowering. Every
reachable nonlocal leaf requires explicit `ExternalStorageProvenance`, including
its owner source and immutable revision. Duplicate evidence, local/external
ownership conflicts, missing evidence, cyclic shapes, foreign table types, and
the wrong file kind are typed refusals; neither identity nor a later Rust path
is treated as archive evidence.

Catalog construction, grammar identities, naming/generated-stream assignments,
metadata transitions, authority proof, and later textual resolution remain
authority/assembly obligations upstream of Nomos. This crate neither embeds a
fixture authority nor invents identities to make an incomplete transaction
appear lowerable.

The transitional `SliceOneTransformation` reads only the published `WholeEthos`
positional carrier and constructs the published `WholeLogos` positional carrier.
Its module imports those two typed carriers and the production encodedID
contract.
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
the current structural-codec 0.19 contracts are reserved for canonical
`EncodedForm` carriers.

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
landing values, while InsertAt inserts its following element at an original
Splice-span boundary. Recursive authored Invoke compiles to one internal
judgment whose marker and child Splice emit no marker data; no operational
future marker survives.

When a recursive declaration exists, whole-universe source-graph preflight
rejects application edges, enumeration sharing, and cycles before NameTree
authentication or output. Identity references to enumerations traverse;
identity references to newtypes, builtins, absent declarations, and external
declarations are leaves. The NameTree boundary then authenticates and plans
before item evaluation. Derived-name realization and the complete output tree
are pure plan operations; the daemon commits planned names and output storage
atomically only after successful evaluation. This two-phase boundary is
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

Transformers are data. The current authored `TemplateFuture` algebra is
`Realize`, `Invoke`, `Splice`, and `InsertAt`. Recursive source syntax remains
an ordinary self-`Invoke`; compilation alone replaces it with
`TemplateFuture::RecursiveInvoke { payload: Box<RecursiveCallJudgment> }`,
which has no authored spelling. The retained three-operation `Escape` enum is
legacy `MacroPackage` evidence only. Any surviving name-transform intent must
remain typed data until TextualForm evaluation.

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
`TemplateFuture::Invoke` stores the invoked transformer's durable chain.
Declarations are `Named`, per-section `Structural`, or finite-owned-tree
`Recursive`.

`[delegated-assent]` The designated Claude advisor accepted the po2.19 authored
algebra, recursive compilation equations, whole-source preflight, and evaluator
semantics. This is delegated design-review traceability, not a statement of
psyche conviction or intent.

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
