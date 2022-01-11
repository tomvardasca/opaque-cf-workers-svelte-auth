
use argon2::Params;
use digest::Digest;
use generic_array::GenericArray;
use opaque_ke::{
    ciphersuite::CipherSuite, errors::InternalPakeError, hash::Hash, slow_hash::SlowHash,
};
use generic_array::typenum::Unsigned;



pub struct ArgonSlowHash;

impl<D: Hash> SlowHash<D> for ArgonSlowHash {
    fn hash(
        input: GenericArray<u8, <D as Digest>::OutputSize>,
    ) -> std::result::Result<Vec<u8>, InternalPakeError> {
        let mut output = vec![0u8; <D as Digest>::OutputSize::USIZE];
        let params = Params::new(2048, 4, 2, Some(<D as Digest>::OutputSize::USIZE)).unwrap();

        let argon = argon2::Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            params
        );
        argon
            .hash_password_into(&input, &[0; argon2::MIN_SALT_LEN], &mut output).unwrap();
        Ok(output.to_vec())
    }
}

// The ciphersuite trait allows to specify the underlying primitives
// that will be used in the OPAQUE protocol
#[allow(dead_code)]
pub struct AuthenticationOpaque;
impl CipherSuite for AuthenticationOpaque {
    type Group = curve25519_dalek::ristretto::RistrettoPoint;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDH;
    type Hash = sha2::Sha512;
    type SlowHash = ArgonSlowHash;
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
