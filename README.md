# core-nomos

The stringless **encoded form of Nomos**, the macro/transformation language. A
macro is typed data — never text, never a Rust macro — that lowers the Ethos
encoded form into the Logos encoded form. This crate is the capstone of the
five-language pipeline:

```
Ethos text → Ethos encoded form → Nomos macros → Logos encoded form → TextualRust → generated Rust
```

Generated programs compiling and passing behavior tests are the acceptance surface;
rendered-source equality is not an oracle.

## The shape in one screen

```rust
use core_nomos::MacroPackage;
use core_ethos::TextualEthos;
use core_ethos::fixture::COMMIT_SEQUENCE;
use name_table::{IdentifierNamespace, NameTable};
use textual_rust::RustSource;

// Ethos text → EncodedEthos
let textual = TextualEthos::fixture()?;
let mut ethos_names = NameTable::new(IdentifierNamespace::Schema);
let value = textual.decode(COMMIT_SEQUENCE, "CommitSequence.{ Integer }", &mut ethos_names)?;
let ethos = core_ethos::EncodedEthos::new(vec![core_ethos::EncodedDeclaration::public(value)]);

// EncodedEthos → Nomos macros → encoded Logos form
let lowering = MacroPackage::wire_fixture()?.apply(&ethos, &ethos_names)?;

// encoded logos form → TextualRust → generated Rust
let rust = RustSource::project_item(&lowering.items[0], &lowering.names)?;
```

## What it is

- **Two macro kinds** — *named* (dispatched by minted `MacroIdentity`; an unknown
  named invocation is an error) and *structural* (per-section defaults, selected by
  an Ethos declaration's kind).
- **Stateful at rest** — a `MacroPackage` is a durable, archivable,
  content-identified registry of macros as data, carrying its own authoring
  `NameTable` sibling (excluded from the content identity, so it is rename-stable
  and portable).
- **A closed template escape algebra** — `Realize` / `Invoke` / `Splice`. A
  `NameTransform` is typed intent carried by `Realize`, not a fourth escape; its
  name work occurs only at the NameTable/emission boundary.
- **A typed engine** — `MacroPackage::apply` converts Ethos encoded forms to Logos
  encoded forms without string manipulation in its macro transform: named
  invocations resolve or error loudly, structural defaults cover plain declarations,
  recursive invocation is bounded by cycle rejection, and the NameTable/emission
  boundary composes the Ethos compatibility slice into a Logos-owned table.
  The exact pre-identity dependency still spells that slice
  `IdentifierNamespace::Schema`; this crate exports no alias. Identifiers retain
  their namespace tag, and generated names allocate only in Logos.

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

`tests/pipeline.rs`, `tests/enriched.rs`, and `tests/prelude.rs` exercise typed
lowering, deterministic field-name derivation, class selection, and valid Rust
projection without rendered-source fixture comparison. The separate
`language-engine-witness` process test is the working-program gate: after it is
pinned to this bootstrap revision, it must compile emitted Rust and pass its public
behavior tests. A rename preserves encoded-form identity while changing projected
text.

See `ARCHITECTURE.md` for the design decisions and flagged forks.

The Capsule surface consumes
`content-identity@f1f9c6efc828acaefd0f751550cd40389d312bf5` under the dependency
name `capsule-content-identity` and
`protos@1435c9aeb7f24e811aca670101e355ff26818ae2`. It consumes the published core
producers at `core-ethos@b9db643a853b1f52f10a4100a791d5dbc8c7240d` and
`core-logos@47ede529e4440674e79d748c6160c051ffb62acd`. The legacy package/archive
identity and flat name-table graph remain on their existing typed revisions.

## Build

The Nix flake is the gate:

```
nix flake check      # build · test · clippy · fmt · doc
```

Licensed under MIT OR Apache-2.0.
