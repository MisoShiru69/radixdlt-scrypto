use crate::resource::*;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::*;
use scrypto::math::Decimal;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum ProofRuleResource {
    NonFungible(NonFungibleAddress),
    Resource(ResourceDefId),
    FromComponent(Vec<usize>),
}

impl From<NonFungibleAddress> for ProofRuleResource {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        ProofRuleResource::NonFungible(non_fungible_address)
    }
}

impl From<ResourceDefId> for ProofRuleResource {
    fn from(resource_def_id: ResourceDefId) -> Self {
        ProofRuleResource::Resource(resource_def_id)
    }
}

impl From<Vec<usize>> for ProofRuleResource {
    fn from(path: Vec<usize>) -> Self {
        ProofRuleResource::FromComponent(path)
    }
}

/// Authorization Rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, Describe, TypeId, Encode, Decode)]
pub enum ProofRule {
    This(ProofRuleResource),
    SomeOfResource(Decimal, ProofRuleResource),
    CountOf { count: u8, resources: Vec<ProofRuleResource> },
    AllOf(Vec<ProofRule>),
    OneOf(Vec<ProofRule>),
}

impl ProofRule {
    pub fn or(self, other: ProofRule) -> Self {
        match self {
            ProofRule::This(_) => ProofRule::OneOf(vec![self, other]),
            ProofRule::SomeOfResource(_, _) => ProofRule::OneOf(vec![self, other]),
            ProofRule::AllOf(_) => ProofRule::OneOf(vec![self, other]),
            ProofRule::OneOf(mut rules) => {
                rules.push(other);
                ProofRule::OneOf(rules)
            }
            ProofRule::CountOf { count: _, resources: _ } => ProofRule::OneOf(vec![self, other]),
        }
    }
}

impl From<NonFungibleAddress> for ProofRule {
    fn from(non_fungible_address: NonFungibleAddress) -> Self {
        ProofRule::This(non_fungible_address.into())
    }
}

impl From<ResourceDefId> for ProofRule {
    fn from(resource_def_id: ResourceDefId) -> Self {
        ProofRule::This(resource_def_id.into())
    }
}

#[macro_export]
macro_rules! self_ref {
    ($path_str:expr) => ({
        let path: Vec<usize> = $path_str.split('/').map(|s| s.parse::<usize>().unwrap()).collect();
        let auth: ::scrypto::resource::ProofRuleResource = path.into();
        auth
    });
}

#[macro_export]
macro_rules! any_of {
    ($resource:expr) => ({
        let resource: ::scrypto::resource::ProofRuleResource = $resource.into();
        ::scrypto::resource::ProofRule::This(resource)
    });
    ($left:expr, $($right:expr),+) => ({
        let resource: ::scrypto::resource::ProofRuleResource = $left.into();
        let auth = ::scrypto::resource::ProofRule::This(resource);
        auth.or(any_of!($($right),+))
    });
}

#[macro_export]
macro_rules! all_of {
    ($resource:expr) => ({
        let resource: ::scrypto::resource::ProofRuleResource = $resource.into();
        ::scrypto::resource::ProofRule::This(resource)
    });
    ($left:expr, $($right:expr),+) => ({
        let resource: ::scrypto::resource::ProofRuleResource = $left.into();
        let auth = ::scrypto::resource::ProofRule::This(resource);
        ::scrypto::resource::ProofRule::AllOf(vec![auth, all_of!($($right),+)])
    });
}

#[macro_export]
macro_rules! min_n_of {
    ($count:expr, $resource:expr) => (
        ProofRule::CountOf {
            count: $count,
            resources: vec![$resource.into()]
        }
    );
    ($count:expr, $left:expr, $($right:expr),+) => ({
        let mut auth = min_n_of!($count, $($right),+);
        match auth {
            // TODO: retain original order
            ProofRule::CountOf { count, mut resources } => {
                resources.push($left.into());
                ProofRule::CountOf { count, resources }
            },
            _ => panic!("Should never get here.")
        }
    })
}
#[macro_export]
macro_rules! amount_of {
    ($amount:expr, $resource:expr) => {
        ProofRule::SomeOfResource($amount, $resource.into())
    };
}
