# core-nomos

The stringless **encoded form of Nomos**, the macro/transformation language. A
macro is typed data — never text, never a Rust macro — that lowers the schema
encoded form into the logos encoded form. This crate is the capstone of the
five-language pipeline:

```
schema text → schema encoded form → Nomos macros → logos encoded form → TextualRust → generated Rust
```

Generated programs compiling and passing behavior tests are the acceptance surface;
rendered-source equality is not an oracle.

## The shape in one screen

```rust
use core_nomos::MacroPackage;
use core_schema::TextualSchema;
use core_schema::fixture::COMMIT_SEQUENCE;
use name_table::NameTable;
use textual_rust::RustSource;

// schema TEXT → CoreSchema
let textual = TextualSchema::fixture()?;
let mut schema_names = NameTable::new();
let value = textual.decode(COMMIT_SEQUENCE, "CommitSequence.{ Integer }", &mut schema_names)?;
let schema = core_schema::CoreSchema::new(vec![core_schema::CoreDeclaration::public(value)]);

// CoreSchema → Nomos macros → CoreLogos (+ the extended, continuous NameTable)
let lowering = MacroPackage::wire_fixture().apply(&schema, &schema_names)?;

// CoreLogos → TextualRust → generated Rust
let rust = RustSource::project_item(&lowering.items[0], &lowering.names)?;
```

## What it is

- **Two macro kinds** — *named* (dispatched by minted `MacroIdentity`; an unknown
  named invocation is an error) and *structural* (per-section defaults, selected by
  a schema declaration's kind).
- **Stateful at rest** — a `MacroPackage` is a durable, archivable,
  content-identified registry of macros as data, carrying its own authoring
  `NameTable` sibling (excluded from the content identity, so it is rename-stable
  and portable).
- **A closed template escape algebra** — `Realize` / `Invoke` / `Splice`. A
  `NameTransform` is typed intent carried by `Realize`, not a fourth escape; its
  name work occurs only at the NameTable/emission boundary.
- **A typed engine** — `MacroPackage::apply` converts schema encoded forms to logos
  encoded forms without string manipulation in its macro transform: named
  invocations resolve or error loudly, structural defaults cover plain declarations,
  recursive invocation is bounded by cycle rejection, and the NameTable/emission
  boundary extends the identifier space (schema indices preserved, logos names
  appended).

## What it is not (yet)

**TextualNomos is deferred.** Its escape spelling, meta-type text spellings, and
Nomos delimiters remain in the psyche's review-later pile. This crate parses and
prints no Nomos text: a macro is authored as data.

## Verification

`tests/pipeline.rs`, `tests/enriched.rs`, and `tests/prelude.rs` exercise typed
lowering, deterministic field-name derivation, class selection, and valid Rust
projection without rendered-source fixture comparison. The separate
`language-engine-witness` process test is the working-program gate: after it is
pinned to this bootstrap revision, it must compile emitted Rust and pass its public
behavior tests. A rename preserves encoded-form identity while changing projected
text.

See `ARCHITECTURE.md` for the design, the rulings, and the flagged forks.

## Build

The Nix flake is the gate:

```
nix flake check      # build · test · clippy · fmt · doc
```

Licensed under MIT OR Apache-2.0.
