# core-nomos

The Nomos transformation crate. Its conforming bootstrap path is a direct typed
transformation from an authority-sealed Ethos transaction to the full-chain
Logos carrier. A broader `MacroPackage` engine remains as legacy flat-identifier
evidence and is not the production path.

```
Ethos text
  → BootstrapReader<A> + naming authority
  → PreparedBootstrapTransaction<A>
  → BootstrapSliceOneLowering
  → WholeLogos
  → structural textual form
```

Generated programs compiling and passing behavior tests are the acceptance surface;
rendered-source equality is not an oracle.

## The shape in one screen

```rust,ignore
use core_ethos::bootstrap::{
    BootstrapNamingAuthority, BootstrapReader, PreparedBootstrapTransaction,
};
use core_logos::WholeLogos;
use core_nomos::BootstrapSliceOneLowering;

fn lower<A: BootstrapNamingAuthority>(
    reader: &BootstrapReader<A>,
    transaction: &PreparedBootstrapTransaction<A>,
) -> WholeLogos {
    BootstrapSliceOneLowering::new()
        .lower(reader, transaction)
        .expect("supported Slice One meaning")
}
```

The matching reader revalidates authority authenticity and all prepared-model
invariants immediately before lowering. Nomos accepts no draft, decoded document,
NameTable, or text, and it never manufactures the catalog, grammar identities,
naming assignments, metadata transition, or authority proof. Those are inputs to
the authority-side reader that seals the transaction.

The Slice One boundary lowers canonically ordered Nexus types and role-free
Interface support types. It preserves complete identities, recursive Shape
applications, and unit, unary, or nonempty product variants. Traits, Interface
role relations, Stream declarations, Sema tables, and local Trait requirements
refuse with distinct typed errors carrying the exact unsupported identities.
The transitional `SliceOneTransformation` over `WholeEthos` remains available
for older witnesses while they migrate.

## Authored transformer seal

`AuthoredTransformerDeclaration` is the durable, stringless value between the
TextualNomos boundary and content sealing:

```
.nomos text
  → raw discovery + structural decode
  → AuthoredTransformerDeclaration
  → AuthoredTransformerSet
  → SealedNomosCapsule + AuthenticatedNameTreeProjection
```

The TextualNomos loader validates durable translator receipts before producing
the source-neutral carrier. Declarations and input bindings retain complete
translator-issued Universal encodedID chains, while `Invoke` retains the
invoked transformer's complete durable identity. No package-local
`MacroIdentity` exists in this phase.

The result is a typed Logos skeleton with typed
`Realize`/`Invoke`/`Splice`/`InsertAt` positions, never a string template.
Transformers are `Named`, per-section `Structural`, or finite-owned-tree
`Recursive`. Recursive authored spelling remains `Invoke`; the compiled
recursive judgment is internal and never appears in TextualNomos. `InsertAt`
targets one preceding `Splice` and inserts its following element at an original
span boundary. Every literal
Logos name position — paths, attributes, type references, fields, variants, and
generics — also retains a complete encodedID chain. Rust structs expose typed
positional accessors; field spellings are not authored data.

`SealedNomosCapsule` archives the canonical `AuthoredTransformerSet` and hashes
only those bytes. `AuthenticatedNameTreeProjection` is a separate archive of the
version, bound Capsule identity, and canonically sorted reachable full-chain
spellings under a domain-typed integrity digest. Operational rename advances
only this projection; it preserves the immutable Capsule bytes and identity.
Live slots, automatic repoint, and daemon persistence are deliberately outside
this crate's seal.

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

The obsolete structural-codec 0.6 `EncodedConversion` implementation over
flat `EncodedEthos`/`Vec<EncodedItem>` has been removed. The retained legacy
engine is called through its inherent `apply` methods. Structural-codec 0.19
conversion contracts require the canonical language `EncodedForm` carriers;
the flat execution records are deliberately not adapted into a parallel
conversion universe.

`[delegated-assent]` The designated Claude advisor accepted the po2.19 authored
algebra, recursive compilation, source-graph preflight, and evaluator contract.
This records delegated review and does not claim psyche conviction or intent.

The legacy graph carries flat identifiers, stored or derived field names, and
string-bearing evidence. Those mechanics are not precedents for future Nomos
coverage.

## Legacy Capsule pass-through

`capsule_from_issued_hash` is the kind-fixed Nomos pass-through into
`protos::Capsule<protos::Nomos, Pin>`. The caller supplies both the
`ContentAddressedHash` and opaque complete NameTree pin. `core-nomos` does not
derive that caller-issued hash from `MacroPackage`,
verify content correspondence, inspect or compose the pin, or treat the current
package identity as a whole-Capsule identity. Production authored Nomos uses
`SealedNomosCapsule` instead.

`MacroPackage::content_identity` remains the established package API, including
its current selection boundary. The Capsule pass-through does not reinterpret or
replace it. Flat `Identifier` and `NameTable` state remains explicit migration
debt rather than a nested encodedID-chain claim.

## Verification

`tests/authored_stage.rs` proves archive round trips preserve nested declaration,
binding, invocation, and Logos-literal chains; it also witnesses typed refusal
for wrong roots, duplicate bindings, and undeclared escape bindings.
`tests/textual_nomos_authority.rs` proves receipt-backed sealing, rename-stable
Capsule bytes, versioned projection change, full-chain render/resolve/reseal,
and ancestor-chain identity movement. `tests/textual_nomos_manifest.rs` proves
construction and traversal order do not change the seal.
`tests/textual_nomos.rs` proves exact recursive/`InsertAt` canonical text,
children-before-parent evaluation, leaf behavior, and whole-universe refusal
of application edges, sharing, and cycles.
`tests/slice_one.rs` proves the direct positional mapping and complete-chain
preservation. `tests/slice_one_boundary.rs` mechanically excludes the legacy
NameTable, macro, generation, prelude, rendering, and string surfaces from that
module. The separate `language-engine-witness` compiles and runs emitted Rust as
the working-program gate.

`tests/pipeline.rs`, `tests/enriched.rs`, and `tests/prelude.rs` cover only the
legacy engine. Their field-name derivation and legacy Rust projection assertions
are regression evidence, not production-contract proof.

See `ARCHITECTURE.md` for the design decisions and flagged forks.

Exact producer revisions live in `Cargo.toml` and `Cargo.lock`. The canonical
Ethos and Logos crates expose both the full-chain whole carriers and the
retained legacy execution data, so the manifest carries no parallel aliases or
second structural-codec type universe.

## Build

The Nix flake is the gate:

```
nix flake check      # build · test · clippy · fmt · doc
```

Licensed under MIT OR Apache-2.0.
