# core-nomos

The stringless **Core of Nomos**, the macro/transformation language. A macro is
typed data — never text, never a Rust macro — that lowers a stringless
`CoreSchema` into stringless `CoreLogos`. This crate is the capstone that proves
the psyche's five-language pipeline end to end:

```
schema TEXT → CoreSchema → Nomos macros → CoreLogos → TextualRust → generated Rust
```

against the **real** goldens — macro-produced logos lowers to the exact Rust
`schema-rust` already emits, byte for byte. That byte-exact generated Rust is the
acceptance oracle.

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
- **A closed template escape algebra** — `Realize` / `Invoke` / `Splice`. Name
  synthesis is a `NameTransform` inside `Realize`, not a fourth escape.
- **A typed engine** — `MacroPackage::apply` converts `CoreSchema` → `CoreLogos`
  entirely outside text: named invocations resolve or error loudly, structural
  defaults cover plain declarations, recursive invocation is bounded by cycle
  rejection, and the NameTable is extended continuously (schema indices preserved,
  logos names appended).

## What it is not (yet)

**TextualNomos is deferred.** The `$` / `<< >>` escape sigils, the meta-type text
spellings, and Nomos's own delimiters are in the psyche's review-later pile. This
crate parses and prints no Nomos text: a macro is authored as data.

## The proof

`tests/pipeline.rs` lowers real schema text to real on-disk `textual-rust`
provenance goldens byte-for-byte (the `runner_generated.rs` newtypes via the plain
package; `spirit_generated.rs`'s `RecordIdentifier`, `Topic`, `Entry`, `Query` via
the wire package), runs the illustrative sample pair end to end, and asserts the
hash discipline (a rename moves neither Core identity while the output text
changes).

See `ARCHITECTURE.md` for the design, the rulings, and the flagged forks.

## Build

The Nix flake is the gate:

```
nix flake check      # build · test · clippy · fmt · doc
```

Licensed under MIT OR Apache-2.0.
