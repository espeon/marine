use songbird::{input::AuxMetadata, typemap::TypeMapKey};

pub struct Metadata;

impl TypeMapKey for Metadata {
    type Value = AuxMetadata;
}
