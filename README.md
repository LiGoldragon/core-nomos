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

## Authored transformer phase

`AuthoredTransformerDeclaration` is the durable, stringless value between the
TextualNomos boundary and package sealing:

```
.nomos text
  → raw discovery + structural decode
  → AuthoredTransformerDeclaration
  → atomic package seal
  → MacroDefinition
```

The textual decoder is the next train stage and is not yet implemented here.
The carrier it targets is implemented: declarations and input bindings retain
complete translator-issued Universal encodedID chains, while `Invoke` retains
the invoked transformer's complete durable identity. No package-local
`MacroIdentity` exists in this phase. A later seal sees the complete resolved
declaration set, refuses duplicate or unresolved targets before mutation, and
only then rebinds durable invocation targets into the sealed execution table.

`AuthoredResultSkeleton` means a typed Logos skeleton with typed
`Realize`/`Invoke`/`Splice` positions, never a string template. Every literal
Logos name position — paths, attributes, type references, fields, variants, and
generics — also retains a complete encodedID chain. Rust structs expose typed
positional accessors; field spellings are not authored data.

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

**The TextualNomos decoder is not wired yet.** The approved base door uses the
Standard protos profile and reserved applications
`Realize.<binding>`/`Splice.<binding>`/`Invoke.<transformer>`. This revision
publishes its phase-stable output contract; it does not yet parse or print
Nomos text, allocate translator identities, resolve manifests, or seal an
execution package.

## Verification

`tests/authored_stage.rs` proves archive round trips preserve nested declaration,
binding, invocation, and Logos-literal chains; it also witnesses typed refusal
for wrong roots, duplicate bindings, and undeclared escape bindings.
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
