use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::model::*;
use crate::data::scrypto::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::{Arbitrary, Result, Unstructured};
use sbor::rust::prelude::*;
use sbor::*;
use utils::copy_u8_array;

pub const NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH: usize = 64;

/// Marks the rust type that represents a non-fungible id, of any kind (i.e. String, Integer, Bytes and RUID).
pub trait IsNonFungibleLocalId: Into<NonFungibleLocalId> {
    fn id_type() -> NonFungibleIdType;
}

impl IsNonFungibleLocalId for StringNonFungibleLocalId {
    fn id_type() -> NonFungibleIdType {
        NonFungibleIdType::String
    }
}
impl IsNonFungibleLocalId for IntegerNonFungibleLocalId {
    fn id_type() -> NonFungibleIdType {
        NonFungibleIdType::Integer
    }
}
impl IsNonFungibleLocalId for BytesNonFungibleLocalId {
    fn id_type() -> NonFungibleIdType {
        NonFungibleIdType::Bytes
    }
}
impl IsNonFungibleLocalId for RUIDNonFungibleLocalId {
    fn id_type() -> NonFungibleIdType {
        NonFungibleIdType::RUID
    }
}

/// Marks the rust type that represents a non-fungible id, of non-auto-generated kind (i.e. String, Integer and Bytes).
pub trait IsNonAutoGeneratedNonFungibleLocalId: IsNonFungibleLocalId {}

impl IsNonAutoGeneratedNonFungibleLocalId for StringNonFungibleLocalId {}
impl IsNonAutoGeneratedNonFungibleLocalId for IntegerNonFungibleLocalId {}
impl IsNonAutoGeneratedNonFungibleLocalId for BytesNonFungibleLocalId {}

impl TryFrom<String> for NonFungibleLocalId {
    type Error = ContentValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(StringNonFungibleLocalId::new(value)?.into())
    }
}

impl From<u64> for NonFungibleLocalId {
    fn from(value: u64) -> Self {
        IntegerNonFungibleLocalId::new(value).into()
    }
}

impl TryFrom<Vec<u8>> for NonFungibleLocalId {
    type Error = ContentValidationError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(BytesNonFungibleLocalId::new(value)?.into())
    }
}

impl From<[u8; 32]> for NonFungibleLocalId {
    fn from(value: [u8; 32]) -> Self {
        Self::RUID(value.into())
    }
}

/// Represents the local id of a non-fungible.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NonFungibleLocalId {
    /// String matching `[_0-9a-zA-Z]{1,64}`.
    ///
    /// Create using `NonFungibleLocalId::string(...).unwrap()`.
    String(StringNonFungibleLocalId),
    /// Unsigned integers, up to u64.
    ///
    /// Create using `NonFungibleLocalId::integer(...)`.
    Integer(IntegerNonFungibleLocalId),
    /// Bytes, of length between 1 and 64.
    ///
    /// Create using `NonFungibleLocalId::bytes(...).unwrap()`.
    Bytes(BytesNonFungibleLocalId),
    /// RUID, v4, variant 1, big endian. See https://www.rfc-editor.org/rfc/rfc4122
    ///
    /// Create using `NonFungibleLocalId::ruid(...).unwrap()`.
    RUID(RUIDNonFungibleLocalId),
}

impl NonFungibleLocalId {
    pub fn string<T: AsRef<[u8]>>(value: T) -> Result<Self, ContentValidationError> {
        StringNonFungibleLocalId::new(value).map(Self::String)
    }

    pub fn integer(value: u64) -> Self {
        value.into()
    }

    pub fn bytes<T: Into<Vec<u8>>>(value: T) -> Result<Self, ContentValidationError> {
        value.into().try_into()
    }

    pub fn ruid(value: [u8; 32]) -> Self {
        Self::RUID(RUIDNonFungibleLocalId(value))
    }

    pub fn to_key(&self) -> Vec<u8> {
        scrypto_encode(self).expect("Failed to encode non-fungible local id")
    }
}

impl From<StringNonFungibleLocalId> for NonFungibleLocalId {
    fn from(value: StringNonFungibleLocalId) -> Self {
        Self::String(value)
    }
}

impl From<IntegerNonFungibleLocalId> for NonFungibleLocalId {
    fn from(value: IntegerNonFungibleLocalId) -> Self {
        Self::Integer(value)
    }
}

impl From<BytesNonFungibleLocalId> for NonFungibleLocalId {
    fn from(value: BytesNonFungibleLocalId) -> Self {
        Self::Bytes(value)
    }
}

impl From<RUIDNonFungibleLocalId> for NonFungibleLocalId {
    fn from(value: RUIDNonFungibleLocalId) -> Self {
        Self::RUID(value)
    }
}

/// A string matching `[_0-9a-zA-Z]{1,64}`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringNonFungibleLocalId(Vec<u8>);

impl StringNonFungibleLocalId {
    pub fn new<S: AsRef<[u8]>>(id: S) -> Result<Self, ContentValidationError> {
        Self::validate_slice(id.as_ref())?;
        Ok(Self(id.as_ref().to_vec()))
    }

    pub fn validate_slice(slice: &[u8]) -> Result<(), ContentValidationError> {
        if slice.len() == 0 {
            return Err(ContentValidationError::Empty);
        }
        if slice.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
            return Err(ContentValidationError::TooLong);
        }
        for byte in slice {
            let byte = *byte;
            if byte >= b'a' && byte <= b'z'
                || byte >= b'A' && byte <= b'Z'
                || byte >= b'0' && byte <= b'9'
                || byte == b'_'
            {
                continue;
            } else {
                return Err(ContentValidationError::ContainsBadCharacter);
            }
        }

        Ok(())
    }

    pub fn value(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.0.as_slice()) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[cfg(feature = "radix_engine_fuzzing")]
impl<'a> Arbitrary<'a> for StringNonFungibleLocalId {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let charset: Vec<char> =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWZYZ012345678989_"
                .chars()
                .collect();
        let len: u8 = u
            .int_in_range(1..=NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH as u8)
            .unwrap();
        let s: String = (0..len).map(|_| *u.choose(&charset[..]).unwrap()).collect();

        Ok(Self(s.into_bytes()))
    }
}

impl TryFrom<String> for StringNonFungibleLocalId {
    type Error = ContentValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for StringNonFungibleLocalId {
    type Error = ContentValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Unsigned integers, up to u64.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntegerNonFungibleLocalId(u64);

impl IntegerNonFungibleLocalId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl From<u64> for IntegerNonFungibleLocalId {
    fn from(value: u64) -> Self {
        IntegerNonFungibleLocalId::new(value)
    }
}

/// Bytes, of length between 1 and 64.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BytesNonFungibleLocalId(Vec<u8>);

impl BytesNonFungibleLocalId {
    pub fn new(id: Vec<u8>) -> Result<Self, ContentValidationError> {
        let new = Self(id);
        new.validate()?;
        Ok(new)
    }

    pub fn validate(&self) -> Result<(), ContentValidationError> {
        if self.0.len() == 0 {
            return Err(ContentValidationError::Empty);
        }
        if self.0.len() > NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH {
            return Err(ContentValidationError::TooLong);
        }
        Ok(())
    }

    pub fn value(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(feature = "radix_engine_fuzzing")]
impl<'a> Arbitrary<'a> for BytesNonFungibleLocalId {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let len: u8 = u
            .int_in_range(1..=NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH as u8)
            .unwrap();
        let s = (0..len).map(|_| u8::arbitrary(u).unwrap()).collect();

        Ok(Self(s))
    }
}

impl TryFrom<Vec<u8>> for BytesNonFungibleLocalId {
    type Error = ContentValidationError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
/// RUID, v4, variant 1, big endian. See https://www.rfc-editor.org/rfc/rfc4122
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RUIDNonFungibleLocalId([u8; 32]);

impl RUIDNonFungibleLocalId {
    pub fn new(id: [u8; 32]) -> Self {
        Self(id)
    }

    pub fn value(&self) -> &[u8; 32] {
        &self.0
    }
}

impl From<[u8; 32]> for RUIDNonFungibleLocalId {
    fn from(value: [u8; 32]) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentValidationError {
    TooLong,
    Empty,
    ContainsBadCharacter,
}

impl NonFungibleLocalId {
    pub fn id_type(&self) -> NonFungibleIdType {
        match self {
            NonFungibleLocalId::String(..) => NonFungibleIdType::String,
            NonFungibleLocalId::Integer(..) => NonFungibleIdType::Integer,
            NonFungibleLocalId::Bytes(..) => NonFungibleIdType::Bytes,
            NonFungibleLocalId::RUID(..) => NonFungibleIdType::RUID,
        }
    }

    pub fn encode_body_common<X: CustomValueKind, E: Encoder<X>>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        match self {
            NonFungibleLocalId::String(v) => {
                encoder.write_discriminator(0)?;
                encoder.write_size(v.0.len())?;
                encoder.write_slice(v.as_bytes())?;
            }
            NonFungibleLocalId::Integer(v) => {
                encoder.write_discriminator(1)?;
                encoder.write_slice(&v.0.to_be_bytes())?; // TODO: variable length encoding?
            }
            NonFungibleLocalId::Bytes(v) => {
                encoder.write_discriminator(2)?;
                encoder.write_size(v.0.len())?;
                encoder.write_slice(v.0.as_slice())?;
            }
            NonFungibleLocalId::RUID(v) => {
                encoder.write_discriminator(3)?;
                encoder.write_slice(v.value().as_slice())?;
            }
        }
        Ok(())
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut encoder = ScryptoEncoder::new(&mut buffer, 1);
        self.encode_body_common(&mut encoder).unwrap();
        buffer
    }

    pub fn decode_body_common<X: CustomValueKind, D: Decoder<X>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        match decoder.read_discriminator()? {
            0 => {
                let size = decoder.read_size()?;
                let slice = decoder.read_slice(size)?;
                Self::string(slice).map_err(|_| DecodeError::InvalidCustomValue)
            }
            1 => Ok(Self::integer(u64::from_be_bytes(copy_u8_array(
                decoder.read_slice(8)?,
            )))),
            2 => {
                let size = decoder.read_size()?;
                Self::bytes(decoder.read_slice(size)?.to_vec())
                    .map_err(|_| DecodeError::InvalidCustomValue)
            }
            3 => Ok(Self::ruid(copy_u8_array(decoder.read_slice(32)?))),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

//========
// error
//========

/// Represents an error when decoding non-fungible id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleLocalIdError {
    UnknownType,
    InvalidInteger,
    InvalidBytes,
    InvalidRUID,
    ContentValidationError(ContentValidationError),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleLocalIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleLocalIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for NonFungibleLocalId {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for NonFungibleLocalId {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for NonFungibleLocalId {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}

//====================
// binary (manifest)
//====================

impl Categorize<ManifestCustomValueKind> for NonFungibleLocalId {
    #[inline]
    fn value_kind() -> ValueKind<ManifestCustomValueKind> {
        ValueKind::Custom(ManifestCustomValueKind::NonFungibleLocalId)
    }
}

impl<E: Encoder<ManifestCustomValueKind>> Encode<ManifestCustomValueKind, E>
    for NonFungibleLocalId
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ManifestCustomValueKind>> Decode<ManifestCustomValueKind, D>
    for NonFungibleLocalId
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}

impl Describe<ScryptoCustomTypeKind> for NonFungibleLocalId {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::NON_FUNGIBLE_LOCAL_ID_TYPE);

    fn type_data() -> TypeData<ScryptoCustomTypeKind, RustTypeId> {
        well_known_scrypto_custom_types::non_fungible_local_id_type_data()
    }
}

//======
// text
//======

/// We wish to be stricter than `from_str_radix` in order to ensure a canonical format, and in particular:
/// * Not allow + at the start
/// * Not allow leading 0s
/// * Not allow an empty string
fn is_canonically_formatted_integer(digits: &str) -> bool {
    if digits == "0" {
        return true;
    }
    let mut chars = digits.chars();
    // A non-zero integer must start with a digit between 1 and 9
    let first_char = chars.next();
    match first_char {
        None => {
            return false;
        }
        Some('1'..='9') => {}
        _ => {
            return false;
        }
    }
    // The remaining chars must be digits
    for char in chars {
        if !matches!(char, '0'..='9') {
            return false;
        }
    }
    return true;
}

impl FromStr for NonFungibleLocalId {
    type Err = ParseNonFungibleLocalIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let local_id = if s.starts_with("<") && s.ends_with(">") {
            Self::string(s[1..s.len() - 1].to_string())
                .map_err(ParseNonFungibleLocalIdError::ContentValidationError)?
        } else if s.starts_with("#") && s.ends_with("#") {
            let digits = &s[1..s.len() - 1];
            if !is_canonically_formatted_integer(digits) {
                return Err(ParseNonFungibleLocalIdError::InvalidInteger);
            }
            NonFungibleLocalId::integer(
                u64::from_str_radix(&s[1..s.len() - 1], 10)
                    .map_err(|_| ParseNonFungibleLocalIdError::InvalidInteger)?,
            )
        } else if s.starts_with("[") && s.ends_with("]") {
            NonFungibleLocalId::bytes(
                hex::decode(&s[1..s.len() - 1])
                    .map_err(|_| ParseNonFungibleLocalIdError::InvalidBytes)?,
            )
            .map_err(ParseNonFungibleLocalIdError::ContentValidationError)?
        } else if s.starts_with("{") && s.ends_with("}") {
            let chars: Vec<char> = s[1..s.len() - 1].chars().collect();
            if chars.len() == 32 * 2 + 3 && chars[16] == '-' && chars[33] == '-' && chars[50] == '-'
            {
                let hyphen_stripped: String = chars.into_iter().filter(|c| *c != '-').collect();
                if hyphen_stripped.len() == 64 {
                    NonFungibleLocalId::RUID(RUIDNonFungibleLocalId(
                        hex::decode(&hyphen_stripped)
                            .map_err(|_| ParseNonFungibleLocalIdError::InvalidRUID)?
                            .try_into()
                            .unwrap(),
                    ))
                } else {
                    return Err(ParseNonFungibleLocalIdError::InvalidRUID);
                }
            } else {
                return Err(ParseNonFungibleLocalIdError::InvalidRUID);
            }
        } else {
            return Err(ParseNonFungibleLocalIdError::UnknownType);
        };

        Ok(local_id)
    }
}

impl fmt::Display for NonFungibleLocalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NonFungibleLocalId::String(v) => write!(f, "<{}>", v.value()),
            NonFungibleLocalId::Integer(IntegerNonFungibleLocalId(v)) => write!(f, "#{}#", v),
            NonFungibleLocalId::Bytes(BytesNonFungibleLocalId(v)) => {
                write!(f, "[{}]", hex::encode(&v))
            }
            NonFungibleLocalId::RUID(RUIDNonFungibleLocalId(v)) => {
                let hex = hex::encode(v.as_slice());
                write!(
                    f,
                    "{{{}-{}-{}-{}}}",
                    &hex[0..16],
                    &hex[16..32],
                    &hex[32..48],
                    &hex[48..64]
                )
            }
        }
    }
}

impl fmt::Debug for NonFungibleLocalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_fungible_length_validation() {
        // Bytes length
        let validation_result = NonFungibleLocalId::bytes([0; NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH]);
        assert!(matches!(validation_result, Ok(_)));
        let validation_result =
            NonFungibleLocalId::bytes([0; 1 + NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH]);
        assert_eq!(validation_result, Err(ContentValidationError::TooLong));
        let validation_result = NonFungibleLocalId::bytes(vec![]);
        assert_eq!(validation_result, Err(ContentValidationError::Empty));

        // String length
        let validation_result =
            NonFungibleLocalId::string(string_of_length(NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH));
        assert!(matches!(validation_result, Ok(_)));
        let validation_result =
            NonFungibleLocalId::string(string_of_length(1 + NON_FUNGIBLE_LOCAL_ID_MAX_LENGTH));
        assert_eq!(validation_result, Err(ContentValidationError::TooLong));
        let validation_result = NonFungibleLocalId::string("".to_string());
        assert_eq!(validation_result, Err(ContentValidationError::Empty));

        let validation_result =
            NonFungibleLocalId::from_str("{--------------4----8---------------1}");
        assert_eq!(
            validation_result,
            Err(ParseNonFungibleLocalIdError::InvalidRUID)
        );
    }

    fn string_of_length(size: usize) -> String {
        let mut str_buf = String::new();
        for _ in 0..size {
            str_buf.push('a');
        }
        str_buf
    }

    #[test]
    fn test_non_fungible_string_validation() {
        let valid_id_string = "abcdefghijklmnopqrstuvwxyz_ABCDEFGHIJKLMNOPQRSTUVWZYZ_0123456789";
        let validation_result = NonFungibleLocalId::string(valid_id_string.to_owned());
        assert!(matches!(validation_result, Ok(_)));

        test_invalid_char('.');
        test_invalid_char('`');
        test_invalid_char('\\');
        test_invalid_char('"');
        test_invalid_char(' ');
        test_invalid_char('\r');
        test_invalid_char('\n');
        test_invalid_char('\t');
        test_invalid_char('\u{0000}'); // Null
        test_invalid_char('\u{0301}'); // Combining acute accent
        test_invalid_char('\u{2764}'); // ❤
        test_invalid_char('\u{000C}'); // Form feed
        test_invalid_char('\u{202D}'); // LTR override
        test_invalid_char('\u{202E}'); // RTL override
        test_invalid_char('\u{1F600}'); // :-) emoji
    }

    fn test_invalid_char(char: char) {
        let validation_result = NonFungibleLocalId::string(format!("valid_{}", char));
        assert_eq!(
            validation_result,
            Err(ContentValidationError::ContainsBadCharacter)
        );
    }

    #[test]
    fn test_from_str() {
        // Integers and invalid integers:
        assert_eq!(
            NonFungibleLocalId::from_str("#1#").unwrap(),
            NonFungibleLocalId::integer(1)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("#10#").unwrap(),
            NonFungibleLocalId::integer(10)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("#0#").unwrap(),
            NonFungibleLocalId::integer(0)
        );
        // Non-canonical, invalid integers
        assert_eq!(
            NonFungibleLocalId::from_str("##"),
            Err(ParseNonFungibleLocalIdError::InvalidInteger)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("#+10#"),
            Err(ParseNonFungibleLocalIdError::InvalidInteger)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("#010#"),
            Err(ParseNonFungibleLocalIdError::InvalidInteger)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("# 10#"),
            Err(ParseNonFungibleLocalIdError::InvalidInteger)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("#000#"),
            Err(ParseNonFungibleLocalIdError::InvalidInteger)
        );
        assert_eq!(
            NonFungibleLocalId::from_str("#-10#"),
            Err(ParseNonFungibleLocalIdError::InvalidInteger)
        );
        assert_eq!(
            NonFungibleLocalId::from_str(
                "{1111111111111111-1111111111111111-1111111111111111-1111111111111111}"
            )
            .unwrap(),
            NonFungibleLocalId::ruid([0x11; 32])
        );
        assert_eq!(
            NonFungibleLocalId::from_str("<test>").unwrap(),
            NonFungibleLocalId::string("test").unwrap()
        );
        assert_eq!(
            NonFungibleLocalId::from_str("[010a]").unwrap(),
            NonFungibleLocalId::bytes(vec![1, 10]).unwrap()
        );
    }

    #[test]
    fn test_to_string() {
        assert_eq!(NonFungibleLocalId::integer(0).to_string(), "#0#",);
        assert_eq!(NonFungibleLocalId::integer(1).to_string(), "#1#",);
        assert_eq!(NonFungibleLocalId::integer(10).to_string(), "#10#",);
        assert_eq!(
            NonFungibleLocalId::ruid([
                0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
                0x22, 0x22, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x44, 0x44, 0x44, 0x44,
                0x44, 0x44, 0x44, 0x44,
            ])
            .to_string(),
            "{1111111111111111-2222222222222222-3333333333333333-4444444444444444}",
        );
        assert_eq!(
            NonFungibleLocalId::string("test").unwrap().to_string(),
            "<test>"
        );
        assert_eq!(
            NonFungibleLocalId::bytes(vec![1, 10]).unwrap().to_string(),
            "[010a]"
        );
    }
}
