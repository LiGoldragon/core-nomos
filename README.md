# core-nomos

The Nomos transformation crate. Its conforming first-slice path is a direct
typed transformation from the full-chain Ethos carrier to the full-chain Logos
carrier. A broader `MacroPackage` engine remains as legacy flat-identifier
evidence and is not the production path.

```
Ethos text → WholeEthos → SliceOneTransformation → WholeLogos → structural Rust textual form
```

Generated programs compiling and passing behavior tests are the acceptance surface;
rendered-source equality is not an oracle.

## The shape in one screen

```rust
use core_nomos::SliceOneTransformation;
use slice_core_ethos::WholeEthos;
use slice_core_logos::WholeLogos;

fn lower(ethos: &WholeEthos) -> WholeLogos {
    SliceOneTransformation::new().lower(ethos)
}
```

`SliceOneTransformation` accepts no NameTable or text. It maps the current closed
item vocabulary — one attribute-free, non-generic tuple newtype — positionally,
preserving both complete Universal encodedID chains without resolving,
flattening, deriving, or allocating names. It maps visibility and consumes the
typed empty attribute position. Fields carry no names.

## Legacy MacroPackage evidence

The following surfaces remain implemented for regression and migration work.
They do not widen the production slice:

- **Two legacy macro kinds** — *named* (dispatched by `MacroIdentity`) and
  *structural* (selected by an Ethos declaration's kind).
- **Legacy state at rest** — a `MacroPackage` is a durable, archivable,
  content-identified registry of macros as data, carrying its own authoring
  `NameTable` sibling.
- **A closed template escape algebra** — `Realize` / `Invoke` / `Splice`. A
  `NameTransform` is typed intent carried by `Realize`, not a fourth escape; its
  name work occurs through the legacy `NameTableBoundary`.
- **A broader engine** — `MacroPackage::apply` crosses `NameTableBoundary`,
  which resolves spellings, builds derived names, and allocates legacy
  identifiers. Other legacy surfaces construct and render a prelude and feed
  `textual-rust`. This graph therefore does not satisfy the no-string Nomos
  boundary, even though its template walk is typed.

The legacy graph carries flat identifiers, stored or derived field names, and
string-bearing evidence. Those mechanics are not precedents for future Nomos
coverage.

## Capsule carrier

`capsule_from_issued_hash` is the kind-fixed Nomos pass-through into
`protos::Capsule<protos::Nomos, Pin>`. The caller supplies both the
`ContentAddressedHash` and opaque complete NameTree pin. `core-nomos` does not
create a whole-Nomos encoded carrier, derive a Capsule hash from `MacroPackage`,
verify content correspondence, inspect or compose the pin, or treat the current
package identity as a whole-Capsule identity.

`MacroPackage::content_identity` remains the established package API, including
its current selection boundary. The Capsule pass-through does not reinterpret or
replace it. Flat `Identifier` and `NameTable` state remains explicit migration
debt rather than a nested encodedID-chain claim.

## What it is not (yet)

**TextualNomos is deferred.** Its escape spelling, meta-type text spellings, and
Nomos delimiters remain deferred. This crate parses and
prints no Nomos text: a macro is authored as data.

## Verification

`tests/slice_one.rs` proves the direct positional mapping and complete-chain
preservation. `tests/slice_one_boundary.rs` mechanically excludes the legacy
NameTable, macro, generation, prelude, rendering, and string surfaces from that
module. The separate `language-engine-witness` compiles and runs emitted Rust as
the working-program gate.

`tests/pipeline.rs`, `tests/enriched.rs`, and `tests/prelude.rs` cover only the
legacy engine. Their field-name derivation and legacy Rust projection assertions
are regression evidence, not production-contract proof.

See `ARCHITECTURE.md` for the design decisions and flagged forks.

Exact producer revisions live in `Cargo.toml` and `Cargo.lock`. The manifest
keeps separate dependency aliases for the full-chain slice carriers and the
legacy flat-identifier producers.

## Build

The Nix flake is the gate:

```
nix flake check      # build · test · clippy · fmt · doc
```

Licensed under MIT OR Apache-2.0.
