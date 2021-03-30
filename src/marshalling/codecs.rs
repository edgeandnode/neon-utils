use crate::prelude::*;
use faster_hex;
use primitive_types::U256;

pub trait Decode<T: ?Sized> {
    fn decode(s: &T) -> Result<Self, ()>
    where
        Self: Sized;
}

pub trait Encode {
    fn encode(&self) -> String;
}

impl<const N: usize> Decode<str> for [u8; N] {
    fn decode(s: &str) -> Result<Self, ()>
    where
        Self: Sized,
    {
        profile_method!(decode);

        let mut result = [0; N];
        let mut bytes = s.as_bytes();
        if bytes.starts_with(b"0x") {
            bytes = &bytes[2..];
        }
        faster_hex::hex_decode(bytes, &mut result[..]).map_err(|_| ())?;
        Ok(result)
    }
}

impl Encode for Address {
    fn encode(&self) -> String {
        profile_method!(encode);

        const LEN: usize = 42;
        let mut result = String::with_capacity(LEN);
        result.push_str("0x");
        let mut bytes = [0; 40];
        faster_hex::hex_encode(&self[..], &mut bytes).unwrap();
        result.push_str(std::str::from_utf8(&bytes).unwrap());
        debug_assert!(result.len() == LEN);
        result
    }
}

// This appears like a job for const generics, but I had trouble using
// the derived values. Eg: String::with_capacity(/* what? */) and [0; N*2]
// These error right now, which are likely just limitations of the const
// generics MVP.
impl Encode for Bytes32 {
    fn encode(&self) -> String {
        profile_method!(encode);

        const LEN: usize = 66;
        let mut result = String::with_capacity(LEN);
        result.push_str("0x");
        let mut bytes = [0; 64];
        faster_hex::hex_encode(&self[..], &mut bytes).unwrap();
        result.push_str(std::str::from_utf8(&bytes).unwrap());
        debug_assert!(result.len() == LEN);
        result
    }
}

impl Encode for U256 {
    fn encode(&self) -> String {
        profile_method!(encode);

        format!("{}", self)
    }
}

impl Decode<str> for U256 {
    fn decode(s: &str) -> Result<Self, ()> {
        profile_method!(decode);

        U256::from_dec_str(s).map_err(|_| ())
    }
}

pub fn decode<T: ?Sized, D: Decode<T>>(s: impl AsRef<T>) -> Result<D, ()> {
    Decode::decode(s.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_hex() {
        let mut bytes = Bytes32::default();
        bytes[0] = 1;
        bytes[2] = 2;
        let encoded = bytes.encode();

        assert_eq!(
            "0x0100020000000000000000000000000000000000000000000000000000000000",
            &encoded
        );
        assert_eq!(decode(encoded.as_str()), Ok(bytes));
    }

    #[test]
    fn round_trip_u256() {
        for i in 0..10000u32 {
            if i % 31 != 0 {
                continue;
            }
            let i = U256::from(i);
            let enc = i.encode();
            assert_eq!(Ok(i), decode(enc.as_str()));
        }
    }
}
