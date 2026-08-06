use capsule_content_identity::IdentityHasher;
use name_table::EncodedName;

pub(crate) fn storage_shape_hasher(kind: &[u8]) -> IdentityHasher {
    let mut hasher = IdentityHasher::unprimed();
    hasher.update_length_prefixed(b"protos-sema-stored-shape-v1");
    hasher.update_length_prefixed(kind);
    hasher
}

pub(crate) fn update_count(hasher: &mut IdentityHasher, count: usize) {
    let count = u64::try_from(count).expect("Rust collection length fits the u64 shape format");
    hasher.update_length_prefixed(&count.to_be_bytes());
}

pub(crate) fn update_identity(hasher: &mut IdentityHasher, identity: &EncodedName) {
    hasher.update_length_prefixed(&identity.archive_bytes());
}
