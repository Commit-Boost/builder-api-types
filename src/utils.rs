use alloy::rpc::types::beacon::{BlsPublicKey, BlsSignature};
use blst::min_pk::{PublicKey as BlstPublicKey, Signature as BlstSignature};

pub fn alloy_pubkey_to_blst(pubkey: &BlsPublicKey) -> Result<BlstPublicKey, blst::BLST_ERROR> {
    BlstPublicKey::key_validate(&pubkey.0)
}

pub fn alloy_sig_to_blst(signature: &BlsSignature) -> Result<BlstSignature, blst::BLST_ERROR> {
    BlstSignature::from_bytes(&signature.0)
}

pub fn blst_pubkey_to_alloy(pubkey: &BlstPublicKey) -> BlsPublicKey {
    BlsPublicKey::from_slice(&pubkey.to_bytes())
}

pub mod quoted_variable_list_u64 {
    use serde::{ser::SerializeSeq, Deserializer, Serializer};
    use serde_utils::quoted_u64_vec::{QuotedIntVecVisitor, QuotedIntWrapper};
    use ssz_types::{typenum::Unsigned, VariableList};

    pub fn serialize<S, T>(value: &VariableList<u64, T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Unsigned,
    {
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for &int in value.iter() {
            seq.serialize_element(&QuotedIntWrapper { int })?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<VariableList<u64, T>, D::Error>
    where
        D: Deserializer<'de>,
        T: Unsigned,
    {
        deserializer.deserialize_any(QuotedIntVecVisitor).and_then(|vec| {
            VariableList::new(vec)
                .map_err(|e| serde::de::Error::custom(format!("invalid length: {:?}", e)))
        })
    }
}

pub mod as_str {
    use std::{fmt::Display, str::FromStr};

    use serde::Deserialize;

    pub fn serialize<S, T: Display>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&data.to_string())
    }

    pub fn deserialize<'de, D, T, E>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: FromStr<Err = E>,
        E: Display,
    {
        let s = String::deserialize(deserializer)?;
        T::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use serde::{de::DeserializeOwned, Serialize};
    use serde_json::Value;

    pub fn test_encode_decode<T: Serialize + DeserializeOwned>(d: &str) -> T {
        let decoded = serde_json::from_str::<T>(d).expect("deserialize");

        // re-encode to make sure that different formats are ignored
        let encoded = serde_json::to_string(&decoded).unwrap();
        let original_v: Value = serde_json::from_str(d).unwrap();
        let encoded_v: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(original_v, encoded_v, "encode mismatch");

        decoded
    }
}
