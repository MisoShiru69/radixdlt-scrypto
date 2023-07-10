use crate::blueprints::package::{BlueprintVersion, BlueprintVersionKey};
use crate::ScryptoSbor;
use core::fmt;
use core::fmt::Formatter;
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use radix_engine_common::types::GlobalAddress;
use radix_engine_common::types::PackageAddress;
use radix_engine_derive::ManifestSbor;
use sbor::rust::prelude::*;
use scrypto_schema::{InstanceSchema, KeyValueStoreSchema};
use utils::ContextualDisplay;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum OuterObjectInfo {
    Some { outer_object: GlobalAddress },
    None,
}

impl Default for OuterObjectInfo {
    fn default() -> Self {
        OuterObjectInfo::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ObjectInfo {
    pub global: bool,
    pub blueprint_id: BlueprintId,
    pub blueprint_version: BlueprintVersion,
    pub outer_object: OuterObjectInfo,
    pub features: BTreeSet<String>,
    pub instance_schema: Option<InstanceSchema>,
}

impl ObjectInfo {
    pub fn blueprint_version_key(&self) -> BlueprintVersionKey {
        BlueprintVersionKey {
            blueprint: self.blueprint_id.blueprint_name.clone(),
            version: self.blueprint_version,
        }
    }

    pub fn get_outer_object(&self) -> GlobalAddress {
        match &self.outer_object {
            OuterObjectInfo::Some { outer_object } => outer_object.clone(),
            OuterObjectInfo::None { .. } => {
                panic!("Broken Application logic: Expected to be an inner object but is an outer object");
            }
        }
    }

    pub fn get_features(&self) -> BTreeSet<String> {
        self.features.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct GlobalAddressPhantom {
    pub blueprint_id: BlueprintId,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct KeyValueStoreInfo {
    pub schema: KeyValueStoreSchema,
}

#[derive(Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
pub struct BlueprintId {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum BlueprintHook {
    OnVirtualize,
    OnMove,
    OnDrop,
    OnPersist,
}

impl BlueprintId {
    pub fn new<S: ToString>(package_address: &PackageAddress, blueprint_name: S) -> Self {
        BlueprintId {
            package_address: *package_address,
            blueprint_name: blueprint_name.to_string(),
        }
    }

    pub fn len(&self) -> usize {
        self.package_address.as_ref().len() + self.blueprint_name.len()
    }
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for BlueprintId {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(
            f,
            "{}:<{}>",
            self.package_address.display(*context),
            self.blueprint_name,
        )
    }
}

impl core::fmt::Debug for BlueprintId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}
