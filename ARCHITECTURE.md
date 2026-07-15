# core-nomos — architecture

The stringless **Core of Nomos**, the macro/transformation language. A macro is
typed data that lowers a stringless `CoreSchema` into stringless `CoreLogos`. This
crate is the capstone of the five-language pipeline:

```
schema TEXT → CoreSchema → Nomos macros → CoreLogos → TextualRust → generated Rust
```

and it proves that pipeline end to end against the real goldens: macro-produced
logos lowers to the exact Rust `schema-rust` already emits, byte for byte. That is
the ruling this crate embodies — *macros define the entire schema→logos lowering,
and the currently generated Rust is the acceptance oracle.*

## What is settled here (CoreNomos), and what is deferred (TextualNomos)

**CoreNomos is built here** and is settled: macros as typed data transforming
`CoreSchema` → `CoreLogos`, the two macro kinds, the stateful-at-rest package, the
closed template escape algebra, and the engine.

**TextualNomos is deferred.** Nomos's text surface — the `$` / `<< >>` escape
sigils, the meta-type text spellings, Nomos's own delimiters — sits in the
psyche's non-rejected review-later pile. Nothing in this crate parses or prints
any Nomos text surface. An escape is a *data* node (`template::Escape`); its text
spelling is explicitly not this crate's concern. This boundary is why the crate
depends on no parser and lifts no grammar: a macro is authored as data (see
`fixtures.rs`), exactly as a daemon would load it.

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
  identifiers), so its content identity under `CoreNomosDomain` is rename-stable by
  construction.
- an authoring `NameTable` **sibling**, excluded from the content identity exactly
  as everywhere in the family. This is what makes the package portable: it carries
  its own names, so it can be archived and re-seated without a foreign table.

Daemon/sema-engine seating is later work; this crate provides the portable package
type and its content identity + revision surfaces.

## The closed template escape algebra

A macro's result template is **CoreLogos-shaped data with escape nodes**. Each
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

**No fourth escape.** Name synthesis is not a new primitive but a `NameTransform`
inside `Realize`, reusing name-table's single home of the derived-name walk
(`Name::field_name` / `screaming` / `pascal_case`). This is the psyche's
no-fourth-escape ruling made structural.

## The engine and the one continuous identifier space

`MacroPackage::apply(schema, schema_names) → Lowering { items, names }`. The
returned NameTable begins as `NameTable::extend_from(schema_names)` — every schema
identifier keeps its exact index — and logos-only names (derive paths, leaf type
names, derived field names) append at higher indices. Because interning dedups, a
derived name that reproduces an existing name reuses its identifier: the
continuous space is a genuine runtime operation, not a bookkeeping claim. Every
template literal is authored against the package's own NameTable and re-interned
through it into the extension, so a portable package composes with any schema
table.

The field-name rule (`FieldNameRule::FieldRuleDispatch`) distinguishes an *elided*
field (its schema name equals the `field_name` of its type — re-derive through the
walker) from an *explicitly named* one (keep the schema name), matching schema's
own decode-time Field-rule split.

## The acceptance oracle, realized

`tests/pipeline.rs` proves the whole chain byte-exact against the real
`textual-rust` provenance goldens (copied from `schema-rust @ 87de872`, the corpus
textual-rust proved 153 items against):

- **From real schema TEXT to a real on-disk golden:** `CommitSequence.{ Integer }`
  and `StateDigest.{ Integer }` decoded by `TextualSchema`, lowered by the *plain*
  package, project byte-for-byte to the `runner_generated.rs` newtypes.
- **Byte-exact real goldens, nomos lowering the only new variable:** the wire
  package reproduces `spirit_generated.rs`'s `RecordIdentifier(Integer)` and
  `Topic(String)` (the full three-attribute preamble) and the multi-field structs
  `Entry { topics, kind, description, magnitude }` and `Query { topic, kind }` —
  every field name derived through name-table's walker.
- **The illustrative sample pair** (CommitSequence / DatabaseMarker with the private
  `secret_digest`) runs end to end, clearly labeled sample-not-golden where a shape
  is not in the on-disk corpus (field visibility is a logos concern CoreSchema does
  not carry).
- **Hash discipline:** a rename leaves both the CoreSchema and CoreLogos identities
  unchanged while the projected Rust text changes — names live only in the
  projection.

## Train status

This crate git-pins the green path of the published stack: `content-identity`,
`name-table`, `core-schema`, `core-logos` (runtime) and `textual-rust`,
`structural-codec` (tests), each at an exact rev. The `core-logos` pin string
matches `textual-rust`'s exactly so Cargo unifies a single `core-logos` — required
for the byte-exact interop (two copies would be two incompatible `CoreItem`
types). The Nix flake (`build`/`test`/`clippy`/`fmt`/`doc`) is the durable gate.

## Flagged forks (unruled readings, chosen per the rulings)

- **`MacroIdentity` is a package-minted `u32`, not a content hash.** The corpus
  says "minted identity" without fixing mint-vs-content-hash; a monotonic package
  mint is the reading most consistent with "a macro table keyed on minted identity"
  (a content hash would be *derived*, and would couple every recursive reference to
  the invoked macro's bytes). The *package* carries content identity.
- **`Splice`'s per-element production is specialized to struct fields.** That is the
  only spliced sequence the fixture corpus exercises; generalizing `SpliceElement`
  to variants/attributes is a growth point, left as a real closed-enum sibling.
- **The field-name derivation runs in Nomos AND in `core-schema`'s decode.** The
  shipped `core-schema` stores field names in the Core value (deriving elided names
  at decode); the design corpus places the derivation at the CoreLogos boundary.
  The engine re-derives through the walker into the extended table; because
  interning dedups, the two derivations coincide and the identifier is stable — the
  idempotence is a feature, and the continuous-space test asserts it.
- **Text-spelling never leaks into Core.** The macro-model report's `$` / `<< >>`
  surfaces are absent here by construction; escapes are the data nodes `Realize` /
  `Invoke` / `Splice`.
