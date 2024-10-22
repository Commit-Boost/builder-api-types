use alloy::rpc::types::beacon::{constants::BLS_DST_SIG, BlsPublicKey, BlsSignature};
use blst::BLST_ERROR;
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

use crate::chain::Chain;
use crate::constants::EMPTY_GENESIS_VALIDATORS_ROOT;

use crate::error::BlstErrorWrapper;
use crate::utils::{alloy_pubkey_to_blst, alloy_sig_to_blst};

pub type BlsSecretKey = blst::min_pk::SecretKey;

pub fn verify_bls_signature(
    pubkey: &BlsPublicKey,
    msg: &[u8],
    signature: &BlsSignature,
) -> Result<(), BlstErrorWrapper> {
    let pubkey = alloy_pubkey_to_blst(pubkey)?;
    let signature = alloy_sig_to_blst(signature)?;

    let res = signature.verify(true, msg, BLS_DST_SIG, &[], &pubkey, true);
    if res == BLST_ERROR::BLST_SUCCESS {
        Ok(())
    } else {
        Err(res.into())
    }
}

pub fn sign_message(secret_key: &BlsSecretKey, msg: &[u8]) -> BlsSignature {
    let signature = secret_key.sign(msg, BLS_DST_SIG, &[]).to_bytes();
    BlsSignature::from_slice(&signature)
}

pub fn compute_signing_root(object_root: [u8; 32], signing_domain: [u8; 32]) -> [u8; 32] {
    #[derive(Default, Debug, TreeHash)]
    struct SigningData {
        object_root: [u8; 32],
        signing_domain: [u8; 32],
    }

    let signing_data = SigningData { object_root, signing_domain };
    signing_data.tree_hash_root().0
}

// NOTE: this currently works only for builder domain signatures and
// verifications
// ref: https://github.com/ralexstokes/ethereum-consensus/blob/cf3c404043230559660810bc0c9d6d5a8498d819/ethereum-consensus/src/builder/mod.rs#L26-L29
pub fn compute_domain(chain: Chain, domain_mask: [u8; 4]) -> [u8; 32] {
    #[derive(Debug, TreeHash)]
    struct ForkData {
        fork_version: [u8; 4],
        genesis_validators_root: [u8; 32],
    }

    let mut domain = [0u8; 32];
    domain[..4].copy_from_slice(&domain_mask);

    let fork_version = chain.genesis_fork_version();
    let fd = ForkData { fork_version, genesis_validators_root: EMPTY_GENESIS_VALIDATORS_ROOT };
    let fork_data_root = fd.tree_hash_root();

    domain[4..].copy_from_slice(&fork_data_root[..28]);

    domain
}

pub fn verify_signed_message<T: TreeHash>(
    chain: Chain,
    pubkey: &BlsPublicKey,
    msg: &T,
    signature: &BlsSignature,
    domain_mask: [u8; 4],
) -> Result<(), BlstErrorWrapper> {
    let domain = compute_domain(chain, domain_mask);
    let signing_root = compute_signing_root(msg.tree_hash_root().0, domain);

    verify_bls_signature(pubkey, &signing_root, signature)
}

pub fn sign_builder_message(
    chain: Chain,
    secret_key: &BlsSecretKey,
    msg: &impl TreeHash,
) -> BlsSignature {
    sign_builder_root(chain, secret_key, msg.tree_hash_root().0)
}

pub fn sign_builder_root(
    chain: Chain,
    secret_key: &BlsSecretKey,
    object_root: [u8; 32],
) -> BlsSignature {
    let domain = chain.builder_domain();
    let signing_root = compute_signing_root(object_root, domain);
    sign_message(secret_key, &signing_root)
}

#[cfg(test)]
mod tests {

    use super::compute_domain;
    use crate::{chain::Chain, constants::APPLICATION_BUILDER_DOMAIN};

    #[test]
    fn test_builder_domains() {
        assert_eq!(
            compute_domain(Chain::Mainnet, APPLICATION_BUILDER_DOMAIN),
            Chain::Mainnet.builder_domain()
        );
        assert_eq!(
            compute_domain(Chain::Holesky, APPLICATION_BUILDER_DOMAIN),
            Chain::Holesky.builder_domain()
        );
        assert_eq!(
            compute_domain(Chain::Sepolia, APPLICATION_BUILDER_DOMAIN),
            Chain::Sepolia.builder_domain()
        );
        assert_eq!(
            compute_domain(Chain::Helder, APPLICATION_BUILDER_DOMAIN),
            Chain::Helder.builder_domain()
        );
    }
}