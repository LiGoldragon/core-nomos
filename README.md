# core-nomos

Core Nomos lowers authority-sealed bootstrap Ethos transactions into typed
`WholeLogos`. The sealed reader/transaction pair is the sole input boundary:

```text
BootstrapReader<Authority>
  + PreparedBootstrapTransaction<Authority>
  → BootstrapSliceOneLowering
  → WholeLogos
```

The matching reader revalidates the authority receipt and complete prepared
model immediately before lowering. Core Nomos accepts no draft or text, allocates
no identities, and does not reconstruct another Ethos representation.

## Lowering

`BootstrapSliceOneLowering::lower` transforms the supported Nexus and role-free
Interface type algebra. It preserves complete identities, authority-canonical
declaration order, recursive Shape applications, and unit, unary, and nonempty
product variants. Unsupported Traits, Interface role relations, Streams, and
Sema input return typed errors carrying the relevant identity.

`BootstrapSliceOneLowering::lower_sema` transforms one sealed Sema document into
stored record declarations followed by its table declarations. Each table record
and key must resolve to storage whose exact shape is known. Document-local stored
types derive that shape from the lowered Logos item. Nonlocal stored types require
an `ExternalStorageProvenance` containing the complete Universal identity, its
storage fingerprint, and the published source and revision that owns it. An
encoded identity or Rust spelling alone is never storage compatibility evidence.

Both operations return a complete typed `WholeLogos` value. Text rendering and
generated-source equality are outside this crate's acceptance boundary.

## Verification

`tests/bootstrap.rs` proves authority revalidation, canonical ordering, complete
identity preservation, typed refusal of unsupported meaning, Sema table lowering,
explicit external storage provenance, and storage-shape distinction.
