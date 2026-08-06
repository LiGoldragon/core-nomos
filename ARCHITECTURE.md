# Architecture — core-nomos

## Authority-sealed boundary

The live crate exports one lowering module. Its input is a
`PreparedBootstrapTransaction<Authority>` paired with the exact
`BootstrapReader<Authority>` that can verify its receipt:

```text
authority-selected identities and order
                │
                v
PreparedBootstrapTransaction<Authority>
                +
BootstrapReader<Authority>
                │ validate_transaction
                v
BootstrapSliceOneLowering
                │
                v
           WholeLogos
```

The authority brand is part of the accepted type. Lowering revalidates receipt
authenticity and every prepared-model invariant before inspecting meaning. It
never receives an unsealed draft, creates an identity, resolves a visible name,
or rebuilds an earlier carrier.

## Type lowering

`lower` accepts the supported Nexus and role-free Interface declaration algebra.
Nominal identities and every nested type-reference identity pass through
unchanged. Authority-supplied canonical ordering has already been established by
the prepared transaction and is retained in the emitted `WholeLogos` items.

The lowering is deliberately closed. Traits, Interface memberships, Streams,
Sema bodies, unsupported Shape applications, empty products, and invalid local
binder use produce distinct `BootstrapSliceOneLoweringError` variants. Partial
Logos is never returned.

## Sema lowering and storage evidence

`lower_sema` accepts only a sealed Sema transaction. Stored nominal declarations
are lowered first; table declarations follow them. Each table record and key must
be present in the storage inventory, and each key must lower as a newtype.

Storage compatibility is structural, not nominal. A document-local declaration
derives a `WholeLogosStorageFingerprint` from its complete lowered storage shape.
A reference outside the document is admitted only through
`ExternalStorageProvenance`, which binds:

- the complete Universal identity;
- the exact storage fingerprint;
- a nonempty published producer source and immutable revision.

Conflicting evidence, unused evidence, missing evidence, non-Universal external
identities, and incompatible record/key shapes are typed refusals. The same
external identity may be shared by multiple tables only through one consistent
evidence record.

## Live source boundary

- `src/bootstrap.rs` owns both lowering modes, storage provenance, and their
  typed refusal surface.
- `src/storage_shape.rs` derives storage fingerprints from typed Logos shape.
- `tests/bootstrap.rs` proves the authority, identity, ordering, refusal, and
  storage laws against sealed Interface, Nexus, and Sema transactions.
