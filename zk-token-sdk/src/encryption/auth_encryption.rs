//! Authenticated encryption implementation.
//!
//! This module is a simple wrapper of the `Aes128GcmSiv` implementation.
#[cfg(not(target_os = "solana"))]
use {
    aes_gcm_siv::{
        aead::{Aead, NewAead},
        Aes128GcmSiv,
    },
    rand::{rngs::OsRng, CryptoRng, Rng, RngCore},
    thiserror::Error,
};
use {
    arrayref::{array_ref, array_refs},
    base64::{prelude::BASE64_STANDARD, Engine},
    sha3::{Digest, Sha3_512},
    solana_sdk::{
        derivation_path::DerivationPath,
        instruction::Instruction,
        message::Message,
        pubkey::Pubkey,
        signature::Signature,
        signer::{
            keypair::generate_seed_from_seed_phrase_and_passphrase, EncodableKey, SeedDerivable,
            Signer, SignerError,
        },
    },
    std::{
        convert::TryInto,
        error, fmt,
        io::{Read, Write},
    },
    subtle::ConstantTimeEq,
    zeroize::Zeroize,
};

#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum AuthenticatedEncryptionError {
    #[error("key derivation method not supported")]
    DerivationMethodNotSupported,

    #[error("pubkey does not exist")]
    PubkeyDoesNotExist,
}

struct AuthenticatedEncryption;
impl AuthenticatedEncryption {
    #[cfg(not(target_os = "solana"))]
    fn keygen<T: RngCore + CryptoRng>(rng: &mut T) -> AeKey {
        AeKey(rng.gen::<[u8; 16]>())
    }

    #[cfg(not(target_os = "solana"))]
    fn encrypt(key: &AeKey, balance: u64) -> AeCiphertext {
        let mut plaintext = balance.to_le_bytes();
        let nonce: Nonce = OsRng.gen::<[u8; 12]>();

        // The balance and the nonce have fixed length and therefore, encryption should not fail.
        let ciphertext = Aes128GcmSiv::new(&key.0.into())
            .encrypt(&nonce.into(), plaintext.as_ref())
            .expect("authenticated encryption");

        plaintext.zeroize();

        AeCiphertext {
            nonce,
            ciphertext: ciphertext.try_into().unwrap(),
        }
    }

    #[cfg(not(target_os = "solana"))]
    fn decrypt(key: &AeKey, ct: &AeCiphertext) -> Option<u64> {
        let plaintext =
            Aes128GcmSiv::new(&key.0.into()).decrypt(&ct.nonce.into(), ct.ciphertext.as_ref());

        if let Ok(plaintext) = plaintext {
            let amount_bytes: [u8; 8] = plaintext.try_into().unwrap();
            Some(u64::from_le_bytes(amount_bytes))
        } else {
            None
        }
    }
}

#[derive(Debug, Zeroize)]
pub struct AeKey([u8; 16]);
impl AeKey {
    pub fn new(signer: &dyn Signer, address: &Pubkey) -> Result<Self, SignerError> {
        let message = Message::new(
            &[Instruction::new_with_bytes(*address, b"AeKey", vec![])],
            Some(&signer.try_pubkey()?),
        );
        let signature = signer.try_sign_message(&message.serialize())?;

        // Some `Signer` implementations return the default signature, which is not suitable for
        // use as key material
        if bool::from(signature.as_ref().ct_eq(Signature::default().as_ref())) {
            Err(SignerError::Custom("Rejecting default signature".into()))
        } else {
            Ok(AeKey(signature.as_ref()[..16].try_into().unwrap()))
        }
    }

    pub fn random<T: RngCore + CryptoRng>(rng: &mut T) -> Self {
        AuthenticatedEncryption::keygen(rng)
    }

    pub fn encrypt(&self, amount: u64) -> AeCiphertext {
        AuthenticatedEncryption::encrypt(self, amount)
    }

    pub fn decrypt(&self, ct: &AeCiphertext) -> Option<u64> {
        AuthenticatedEncryption::decrypt(self, ct)
    }
}

impl EncodableKey for AeKey {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Box<dyn error::Error>> {
        let bytes: [u8; 16] = serde_json::from_reader(reader)?;
        Ok(Self(bytes))
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<String, Box<dyn error::Error>> {
        let bytes = self.0;
        let json = serde_json::to_string(&bytes.to_vec())?;
        writer.write_all(&json.clone().into_bytes())?;
        Ok(json)
    }
}

impl SeedDerivable for AeKey {
    fn from_seed(seed: &[u8]) -> Result<Self, Box<dyn error::Error>> {
        const MINIMUM_SEED_LEN: usize = 16;

        if seed.len() < MINIMUM_SEED_LEN {
            return Err("Seed is too short".into());
        }

        let mut hasher = Sha3_512::new();
        hasher.update(seed);
        let result = hasher.finalize();

        Ok(Self(result[..16].try_into()?))
    }

    fn from_seed_and_derivation_path(
        _seed: &[u8],
        _derivation_path: Option<DerivationPath>,
    ) -> Result<Self, Box<dyn error::Error>> {
        Err(AuthenticatedEncryptionError::DerivationMethodNotSupported.into())
    }

    fn from_seed_phrase_and_passphrase(
        seed_phrase: &str,
        passphrase: &str,
    ) -> Result<Self, Box<dyn error::Error>> {
        Self::from_seed(&generate_seed_from_seed_phrase_and_passphrase(
            seed_phrase,
            passphrase,
        ))
    }
}

/// For the purpose of encrypting balances for the spl token accounts, the nonce and ciphertext
/// sizes should always be fixed.
pub type Nonce = [u8; 12];
pub type Ciphertext = [u8; 24];

/// Authenticated encryption nonce and ciphertext
#[derive(Debug, Default, Clone)]
pub struct AeCiphertext {
    pub nonce: Nonce,
    pub ciphertext: Ciphertext,
}
impl AeCiphertext {
    pub fn decrypt(&self, key: &AeKey) -> Option<u64> {
        AuthenticatedEncryption::decrypt(key, self)
    }

    pub fn to_bytes(&self) -> [u8; 36] {
        let mut buf = [0_u8; 36];
        buf[..12].copy_from_slice(&self.nonce);
        buf[12..].copy_from_slice(&self.ciphertext);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<AeCiphertext> {
        if bytes.len() != 36 {
            return None;
        }

        let bytes = array_ref![bytes, 0, 36];
        let (nonce, ciphertext) = array_refs![bytes, 12, 24];

        Some(AeCiphertext {
            nonce: *nonce,
            ciphertext: *ciphertext,
        })
    }
}

impl fmt::Display for AeCiphertext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BASE64_STANDARD.encode(self.to_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        solana_sdk::{signature::Keypair, signer::null_signer::NullSigner},
    };

    #[test]
    fn test_aes_encrypt_decrypt_correctness() {
        let key = AeKey::random(&mut OsRng);
        let amount = 55;

        let ct = key.encrypt(amount);
        let decrypted_amount = ct.decrypt(&key).unwrap();

        assert_eq!(amount, decrypted_amount);
    }

    #[test]
    fn test_aes_new() {
        let keypair1 = Keypair::new();
        let keypair2 = Keypair::new();

        assert_ne!(
            AeKey::new(&keypair1, &Pubkey::default()).unwrap().0,
            AeKey::new(&keypair2, &Pubkey::default()).unwrap().0,
        );

        let null_signer = NullSigner::new(&Pubkey::default());
        assert!(AeKey::new(&null_signer, &Pubkey::default()).is_err());
    }
}
