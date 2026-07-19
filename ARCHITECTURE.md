# core-nomos — architecture

The stringless **encoded form of Nomos**, the macro/transformation language. A
macro is typed data that lowers the schema encoded form into the logos encoded form.
This crate is the capstone of the five-language pipeline:

```
schema text → schema encoded form → Nomos macros → logos encoded form → TextualRust → generated Rust
```

Macros define the entire schema-to-logos lowering. Rendered-source equality is not
an acceptance criterion: generated programs compiling and passing their behavior
tests are the acceptance surface.

## The no-strings invariant

The schema-to-logos transformation is stringless by law. The psyche's ruling is
binding: *"in the nomos transformation (schema to logos), there shall be no string
manipulation/introduction/reading of any kind."* The transformation reads and
writes only typed encoded-form values and the encoded identifiers they carry. It
dispatches on a declaration's Core kind and on `Identifier` indices, and at no point
parses, compares, concatenates, matches, or emits a string. A macro is typed data;
its template is logos-encoded-form data with typed escape nodes; its output is a
logos encoded form. A `NameTransform` is typed intent carried by an escape, never a
spelling the transform reads.

All name derivation and text materialization lives at the NameTable/emission
boundary, which sits outside the transformation. `NameTableBoundary` is the single
home of the derived-name walk; it builds a name's string only as it interns that
name into the continuous identifier space, and text is materialized only when a
value is rendered — `ModuleHead::render` and the TextualRust projection. That string
work is legitimate and required: of the boundary walkers the psyche ruled *"that is
necessary."*

The invariant is exactly this partition — a stringless transformation over typed
encoded-form values and identifiers, with every string confined to the interning
and emission boundary. It is the standing review gate on every macro-engine and
generation-class change: a transformation step that must read or build a string is
misplaced, and its string work belongs at the boundary.

## What is settled here (EncodedNomos), and what is deferred (TextualNomos)

**EncodedNomos is built here** and is settled: macros as typed data transforming
schema encoded forms into logos encoded forms, the two macro kinds, the
stateful-at-rest package, the closed template escape algebra, and the engine.

**TextualNomos is deferred.** Nomos's text surface — its escape spelling,
meta-type text spellings, and delimiters — is genuinely unsettled: an open design
question, not a fixed spelling. Nothing in this crate parses or prints a Nomos text
surface. An escape is a *data* node (`template::Escape`); its spelling is explicitly
not this crate's concern. This boundary is why the macro engine lifts no grammar: a
macro is authored as data (see `fixtures.rs`), exactly as a daemon would load it.

## The fixed module head (`ModuleHead`)

Every generated wire module opens with the same, schema-independent head: the
`// @generated` marker comment, the four scalar type aliases
(`String`/`Integer`/`Boolean`/`Path`), and the cfg-gated NOTA import. That block is
a fixed property of the *module shape* the renderer emits, not of any schema, so it
is Nomos's knowledge, held here (`prelude.rs`) as stringless logos-encoded-form data with a
sibling NameTable. `ModuleHead::render` projects it through the TextualRust codec —
the same `prettyplease` pass the pipeline uses for declarations — so the crate now
takes a `textual-rust` **library** dependency for this one rendering surface. This
does not reintroduce a Nomos grammar: the macro engine stays data-only; only the
generated-Rust *output* head is rendered, exactly as the declarations are.

The head is two projection blocks: the four scalar aliases pack into one
`prettyplease` pass (no blank line between them), and the NOTA import is its own
block; the renderer separates blocks by a blank line, which the render reproduces.
The marker comment sits outside every item, so it is prepended raw — the one
raw-text seam (a recorded lean), with `prettyplease` still the sole formatter of the
item bodies. The projection engine (`logos-engine`) prepends `ModuleHead::render`
ahead of a module's declarations.

## The two macro kinds

Nomos has exactly two dispatch kinds (`identity::MacroKind`):

- **Named** — a macro in the table, reached by explicit `MacroIdentity` (an
  explicit reference or a recursive `Invoke` in another macro's template). *An
  unknown named invocation is an error* (`NomosError::UnknownMacro`).
  `WireAttributes` is named.
- **Structural** — a per-section default (`identity::SectionDefault`), selected by
  a schema declaration's Core kind rather than by name: a newtype declaration
  lowers via the newtype section's default, a struct declaration via the struct
  section's default. `WireNewtype` and the particular-struct macro are structural.

The engine selects a declaration's structural default by `SectionDefault::of_core_type`,
looks the macro up, binds its input from the declaration, and evaluates its
template. A recursive `Invoke` resolves a *named* macro or errors.

## Stateful at rest

Nomos is stateful at rest (the psyche's ruling "5. Yes"). A `MacroPackage` is a
durable, archivable, content-identified value — a loaded-definitions registry as
data:

- `MacroDefinitions` (the content pre-image): the revision, a `MacroIdentity`-keyed
  macro table, and the per-section structural defaults. It is stringless (only
  identifiers), so its content identity under `EncodedNomosDomain` is rename-stable by
  construction.
- an authoring `NameTable` **sibling**, excluded from the content identity exactly
  as everywhere in the family. This is what makes the package portable: it carries
  its own names, so it can be archived and re-seated without a foreign table.

Daemon/sema-engine seating is later work; this crate provides the portable package
type and its content identity + revision surfaces.

## The closed template escape algebra

A macro's result template is **logos-encoded-form data with escape nodes**. Each
position is typed by where it sits — a name position holds an `Identifier`, a type
position a `TypeReference`, an attribute or field position a vector — via the
generic `Scalar<L>` / `Sequence<L>` wrappers, so the template stays strongly typed
while the escape set stays one closed enum (`template::Escape`):

- **Realize** — unquote one bound value at this position, optionally through a
  derived-name transform.
- **Invoke** — recursively invoke another macro by identity; its produced fragment
  is realized (scalar) or spliced (vector) in place. *Recursive invocation is
  required* — WireNewtype invokes WireAttributes — and is bounded by cycle
  rejection (`NomosError::RecursionCycle`).
- **Splice** — expand a bound sequence element by element into the enclosing vector
  (the struct-fields production).

**No fourth escape.** A `NameTransform` is typed intent carried by `Realize`,
not a new primitive. The `NameTableBoundary` executes that intent at the
NameTable/emission boundary, which is the single home of the derived-name walk
(`Name::field_name` / `screaming` / `pascal_case`); the typed Nomos transform never
reads or creates a spelling. This is the psyche's no-fourth-escape ruling made
structural.

## The engine and the one continuous identifier space

`MacroPackage::apply(schema, schema_names) → Lowering { items, names }`. The
`NameTableBoundary` begins the returned NameTable as
`NameTable::extend_from(schema_names)` — every schema identifier keeps its exact
index — and performs all logos-only name allocation (derive paths, leaf type names,
derived field names) at the NameTable/emission boundary. Because interning dedups,
a derived name that reproduces an existing name reuses its identifier: the
continuous space is a genuine runtime operation, not a bookkeeping claim. Every
template literal is authored against the package's own NameTable and re-interned
through that boundary into the extension, so a portable package composes with any
schema table.

The field-name rule (`FieldNameRule::FieldRuleDispatch`) distinguishes an *elided*
field (its schema name equals the `field_name` of its type — re-derive through the
walker) from an *explicitly named* one (keep the schema name), matching schema's
own decode-time Field-rule split.

## Verification boundary

`tests/pipeline.rs` and `tests/enriched.rs` exercise schema decoding, typed Nomos
lowering, identifier continuity, deterministic same-typed-field naming, construct
selection, and projection to valid Rust. They intentionally contain no rendered
Rust fixture comparison. `tests/prelude.rs` verifies the required module-head
surface without treating its formatting as correctness.

The process-level `language-engine-witness` is the working-program gate: it drives
schema, Nomos, and Logos processes, writes the emitted crate under its manifest, and
runs that crate's public behavior tests. The witness must be re-pinned to this
bootstrap revision before it can be acceptance evidence for this revision; until
then its result is historical evidence only.

A rename leaves schema and logos encoded-form identities unchanged while the
projected Rust text changes — names live only in the projection.

## The enriched generation classes (the support surface)

The per-declaration structural defaults lower one logos-encoded item per schema
declaration — the *data* declarations. The wire reference fixtures also carry a schema-derived
*support surface* around those declarations: newtype ergonomics, interface
ergonomics, a thin wire-contract stub, and trace support. These are the enriched
**generation classes** ([`template::GenerationClass`]), a whole-schema layer above
the per-declaration lowering:

- **Class A — `NewtypeErgonomics`:** per data-type newtype, the
  `impl { new / payload / into_payload }` inherent block and the `From<Inner>`
  conversion. The `new` intake is the named contact point `Intake`: the `String`
  scalar leaf takes `impl Into<String>` and constructs through `.into()`; every
  other wrapped type takes its value directly.
- **Class B — `InterfaceErgonomics`:** gated on the interface roots
  (`core_schema::DeclarationRole::InterfaceInput` / `InterfaceOutput`). Per-variant
  constructors that unwrap newtype payloads (the `ConstructorSource` contact point:
  a catalogued-newtype payload is taken as its inner type and wrapped through
  `Newtype::new`), the `From<payload>` conversions, and the cfg-gated `FromStr` /
  `Display` impls.
- **`WireContract` (the ordinary-exchange wire vocabulary):** the `short_header` const
  module, the `SIGNAL_SHORT_HEADER_BYTE_COUNT` byte-count const, the `SignalFrameError`
  enum, and the two route enums — the types the codec speaks. The short-header values
  are derived from each operation's position (root byte 7, variant byte 6), not
  transcribed (LEAN `short-header-derivation-mirrors-legacy`); the short-header
  byte-layout stays the psyche's open **`.38`** review item, mirrored exactly for
  interop.
- **`WireExchangeCodec` (the ordinary-exchange encode/decode bodies):** per interface
  root an `impl` carrying `route`, `short_header`, `route_from_short_header`,
  `encode_signal_frame`, and `decode_signal_frame`, then the request root's
  `SignalOperationHeads` impl. This is what retires the empty letter placeholders
  (former "classes E/F"): the two codec stages are named by their content — the
  vocabulary and the codec over it. The bodies are **behavioral, not source copies of the
  reference fixture**: they mirror the *wire* the hand-written signal contracts speak (an 8-byte
  little-endian short header ahead of an rkyv archive, with a decoded-header-mismatch
  guard) in the shape the modeled statement vocabulary expresses directly — an
  `.ok_or(…)?` in place of an `if … { return … }`, a tuple-variant `UnknownHeader(header)`
  in place of a struct-variant literal — so no struct-literal / early-return node is
  needed. Unported peers (`spirit-judge`, `meta-signal-spirit`) pin `signal-spirit`
  revisions and decode this same wire, so the mirroring is a genuine interop
  requirement, not a convenience. (The streaming/subscription leg — the `Stream`
  construct, `SubscriptionEvent`, the `StreamingFrame` envelope — is under separate
  psyche design and is generated nowhere here.)
- **Class D — `TraceSupport`:** the `SignalObjectName` / `ObjectName` enums with
  their nested-match `name()` bodies, the `pub struct TraceEvent(pub ObjectName);`
  tuple-struct declaration, and the `TraceEvent` impl.

The classes emit the layout-3 item kinds — impl blocks (methods, associated types,
associated consts), functions, consts, const modules — as stringless logos-encoded
items, built directly like the fixed [`ModuleHead`] prelude, every identifier interned into
the one continuous logos NameTable. A package's **enriched selection**
(`MacroPackage::with_selection`, run by `apply_enriched`) is the ordered class list
nomos-engine will later select; the wire and plain fixtures keep an empty selection,
so their behaviour is unchanged, and the selection is outside the content-identity
pre-image.

**Document-order rule (the eventual full-file assembly follows it):** the data
declarations come first, then classes A, B, C, and D; within a class, declarations
and interface roots stay in schema order. `tests/enriched.rs` asserts the class
counts and generation order without comparing a rendered fixture.

**The `TraceEvent` tuple-struct declaration (last class-D gap, closed).** `core-logos`
layout 4 models tuple-field visibility on `Newtype` (`wrapped_visibility:
Visibility`), so class D emits a public newtype whose stored tuple-field visibility
is `Public`, carrying the wire-enum preamble in document order between the
`ObjectName` enum and `impl ObjectName`.

## Train status

This crate git-pins the green path of the published stack: `content-identity`,
`name-table`, `core-schema`, `core-logos` (runtime) and `textual-rust`,
`structural-codec` (tests), each at an exact rev. The `core-logos` pin string
matches `textual-rust`'s exactly so Cargo unifies a single `core-logos`; two
copies would carry incompatible encoded-item types. The Nix flake (`build`/`test`/`clippy`/`fmt`/`doc`) is the durable gate.

## Flagged forks (unruled readings, chosen per the rulings)

- **`MacroIdentity` is a package-minted `u32`, not a content hash.** The corpus
  says "minted identity" without fixing mint-vs-content-hash; a monotonic package
  mint is the reading most consistent with "a macro table keyed on minted identity"
  (a content hash would be *derived*, and would couple every recursive reference to
  the invoked macro's bytes). The *package* carries content identity.
- **`Splice`'s per-element production is specialized to struct fields.** That is the
  only spliced sequence the fixture corpus exercises; generalizing `SpliceElement`
  to variants/attributes is a growth point, left as a real closed-enum sibling.
- **Field-name derivation runs at the NameTable/emission boundary, not in the typed
  Nomos transform.** The shipped `core-schema` still derives its stored field name at
  decode. `NameTableBoundary` independently derives the logos-emission identifier
  from field position and type and interns it into the extended logos table; because
  interning dedups, the two current derivations coincide and the identifier is stable.
  The continuous-space test asserts that idempotence. A future core-schema change is
  outside this lane and requires its own coordinated dependency work.
- **Textual Nomos spelling never enters macro data.** The macro surface remains
  unsettled and has no parser, printer, fixture, or grammar claim here; escapes are
  typed data nodes. Emission-only output literals are projected at the
  NameTable/emission boundary and are not inputs to the Nomos transform.
- **The enriched generation classes are the emission boundary, not macro
  evaluation.** The class-A/B/C/D support surface iterates over schema collections
  (variants into match arms, roots into consts, names into `HEADS` elements) and
  builds output items after typed macro lowering. Its derived-name and output-literal
  work is delegated to `NameTableBoundary`; this keeps text out of the schema→logos
  macro transform without growing the `Realize` / `Splice` template DSL into a second
  copy of the CoreLogos algebra. Trigger to revisit: a class shape that a fixed
  skeleton-with-holes expresses cleanly, or a psyche ruling that the classes must be
  authored as escape-templates.
- **`SignalOperationHeads` is emitted for the request (input) root only.** The reference fixture
  carries one `SignalOperationHeads` impl (`Input`), the request payload's operation
  heads; the codec class follows it. `RequestPayload`, `LogVariant`, the `ExchangeFrame`
  type aliases, and the `into_frame` / `into_reply_frame` envelope constructors remain
  out of scope: the delivered codec is the encode/decode leg (`encode_signal_frame` /
  `decode_signal_frame` over the short-header + rkyv wire), not the frame-envelope
  wrapping. Trigger to revisit: a slice that ports the exchange-envelope surface.
- **The codec bodies are behaviorally specified.** `enriched.rs` asserts their
  signatures and wire logic and projects the assembled module as Rust. The load-bearing
  round-trip proof (generated-encode / hand-written-decode and the reverse) belongs to
  the four-process `language-engine-witness`, where the emitted crate is compiled and
  executed after its producer pins are advanced.
