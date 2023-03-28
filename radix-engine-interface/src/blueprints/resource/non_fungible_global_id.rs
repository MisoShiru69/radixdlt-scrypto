use radix_engine_common::data::scrypto::scrypto_encode;
use crate::address::*;
use crate::constants::*;
use crate::crypto::*;
use crate::data::scrypto::model::*;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::format;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use utils::ContextualDisplay;

/// Represents the global id of a non-fungible.
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct NonFungibleGlobalId(ResourceAddress, NonFungibleLocalId);

impl NonFungibleGlobalId {
    pub const fn new(resource_address: ResourceAddress, local_id: NonFungibleLocalId) -> Self {
        Self(resource_address, local_id)
    }

    /// Returns the resource address.
    pub fn resource_address(&self) -> ResourceAddress {
        self.0
    }

    /// Returns the non-fungible id.
    pub fn local_id(&self) -> &NonFungibleLocalId {
        &self.1
    }

    /// Returns canonical representation of this NonFungibleGlobalId.
    pub fn to_canonical_string(&self, bech32_encoder: &Bech32Encoder) -> String {
        format!("{}", self.display(bech32_encoder))
    }

    /// Converts canonical representation to NonFungibleGlobalId.
    ///
    /// This is composed of `resource_address:id_simple_representation`
    pub fn try_from_canonical_string(
        bech32_decoder: &Bech32Decoder,
        s: &str,
    ) -> Result<Self, ParseNonFungibleGlobalIdError> {
        let parts = s.split(':').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(ParseNonFungibleGlobalIdError::RequiresTwoParts);
        }
        let resource_address = bech32_decoder.validate_and_decode_resource_address(parts[0])?;
        let local_id = NonFungibleLocalId::from_str(parts[1])?;
        Ok(NonFungibleGlobalId::new(resource_address, local_id))
    }
}

//======
// error
//======

/// Represents an error when parsing non-fungible address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleGlobalIdError {
    InvalidResourceAddress(AddressError),
    InvalidNonFungibleLocalId(ParseNonFungibleLocalIdError),
    RequiresTwoParts,
}

impl From<AddressError> for ParseNonFungibleGlobalIdError {
    fn from(err: AddressError) -> Self {
        Self::InvalidResourceAddress(err)
    }
}

impl From<ParseNonFungibleLocalIdError> for ParseNonFungibleGlobalIdError {
    fn from(err: ParseNonFungibleLocalIdError) -> Self {
        Self::InvalidNonFungibleLocalId(err)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleGlobalIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleGlobalIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for NonFungibleGlobalId {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        write!(
            f,
            "{}:{}",
            self.resource_address().display(*context),
            self.local_id()
        )
    }
}

impl fmt::Debug for NonFungibleGlobalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.display(NO_NETWORK))
    }
}

pub trait FromComponent: Sized {
    fn from_component_address(component: ComponentAddress) -> Self;
}

impl FromComponent for NonFungibleGlobalId {
    fn from_component_address(component_address: &ComponentAddress) -> Self {
        let non_fungible_local_id =
            NonFungibleLocalId::bytes(scrypto_encode(component_address).unwrap())
                .unwrap();
        NonFungibleGlobalId::new(
            GLOBAL_OBJECT_TOKEN,
            non_fungible_local_id,
        )
    }
}

pub trait FromPublicKey: Sized {
    fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self;
}

impl FromPublicKey for NonFungibleGlobalId {
    fn from_public_key<P: Into<PublicKey> + Clone>(public_key: &P) -> Self {
        let public_key: PublicKey = public_key.clone().into();
        match public_key {
            PublicKey::EcdsaSecp256k1(public_key) => NonFungibleGlobalId::new(
                ECDSA_SECP256K1_TOKEN,
                NonFungibleLocalId::bytes(hash(public_key.to_vec()).lower_26_bytes()).unwrap(),
            ),
            PublicKey::EddsaEd25519(public_key) => NonFungibleGlobalId::new(
                EDDSA_ED25519_TOKEN,
                NonFungibleLocalId::bytes(hash(public_key.to_vec()).lower_26_bytes()).unwrap(),
            ),
        }
    }
}

//======
// test
//======

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Bech32Decoder;

    #[test]
    fn non_fungible_global_id_canonical_conversion() {
        let dec = Bech32Decoder::for_simulator();
        let enc = Bech32Encoder::for_simulator();

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:<id>",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:<id>"
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:#123#",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:#123#"
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:{8fe4abde-affa-4f99-9a0f-300ec6acb64d}",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:{8fe4abde-affa-4f99-9a0f-300ec6acb64d}"
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:<test>",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:<test>"
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &dec,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:[010a]",
            )
            .unwrap()
            .to_canonical_string(&enc),
            "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:[010a]"
        );
    }

    #[test]
    fn non_fungible_global_id_canonical_conversion_error() {
        let bech32_decoder = Bech32Decoder::for_simulator();
        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn",
            ),
            Err(ParseNonFungibleGlobalIdError::RequiresTwoParts)
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:1:2",
            ),
            Err(ParseNonFungibleGlobalIdError::RequiresTwoParts)
        );

        assert_eq!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:",
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidNonFungibleLocalId(
                ParseNonFungibleLocalIdError::UnknownType
            ))
        );

        assert!(matches!(
            NonFungibleGlobalId::try_from_canonical_string(&bech32_decoder, ":",),
            Err(ParseNonFungibleGlobalIdError::InvalidResourceAddress(_))
        ));

        assert!(matches!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                "3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrs520k2p:#1#",
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidResourceAddress(_))
        ));

        assert!(matches!(
            NonFungibleGlobalId::try_from_canonical_string(
                &bech32_decoder,
                "resource_sim1qxntya3nlyju8zsj8h86fz8ma5yl8smwjlg9tckkqvrsxhzgyn:#notnumber#",
            ),
            Err(ParseNonFungibleGlobalIdError::InvalidNonFungibleLocalId(
                ParseNonFungibleLocalIdError::InvalidInteger
            ))
        ));
    }
}
