//! Authority-sealed bootstrap Ethos lowering into Logos.
//!
//! The live Core Nomos boundary accepts only a
//! [`core_ethos::bootstrap::PreparedBootstrapTransaction`] paired with its
//! matching reader. It allocates no identities, reconstructs no earlier Ethos
//! carrier, and retains authority-selected ordering throughout lowering.

pub mod bootstrap;
mod storage_shape;

pub use bootstrap::{
    BootstrapSliceOneLowering, BootstrapSliceOneLoweringError, ExternalStorageProvenance,
    InterfaceRoleTraitIdentities, StorageProvenanceOwner,
};
