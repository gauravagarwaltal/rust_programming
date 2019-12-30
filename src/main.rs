use hello_macro::HelloMacro;
use hello_macro_derive::HelloMacro;
extern crate poa;
use libp2p::multihash::{encode, Hash};
use serde::{Deserialize, Serialize};
// pub mod keypair::{Keypair, PublicKey};
// pub mod keypair;
// use keypair::{Keypair, PublicKey};
#[derive(HelloMacro)]
struct Pancakes;

pub trait CryptoKeypair<T, U> {
    fn generate() -> T;
    ///Generate from secrete key as byte array
    fn generate_from(secret: &mut [u8]) -> T;
    fn public(keypair: &T) -> U;
    fn secret(keypair: &T) -> Vec<u8>;
    fn sign(keypair: &T, msg: &[u8]) -> Vec<u8>;
}

pub trait Sign<T> {
    fn sign(secret: &T, msg: &[u8]) -> Vec<u8>;
}

pub trait Verify<T> {
    fn verify(public: &T, msg: &[u8], signature: &[u8]) -> bool;
    fn decode(pk_bytes: &[u8]) -> T;
}


#[derive(Debug)]
pub struct Keypair {}

#[derive(Debug)]
pub struct PublicKey {}

#[derive(Debug)]
pub struct SecretKey {}

impl CryptoKeypair<libp2p::identity::ed25519::Keypair, libp2p::identity::ed25519::PublicKey>
    for Keypair
{
    fn generate() -> libp2p::identity::ed25519::Keypair {
        libp2p::identity::ed25519::Keypair::generate()
    }
    fn generate_from(secret_bytes: &mut [u8]) -> libp2p::identity::ed25519::Keypair {
        let secret_key = libp2p::identity::ed25519::SecretKey::from_bytes(secret_bytes);
        libp2p::identity::ed25519::Keypair::from(secret_key.unwrap())
    }
    fn public(
        keypair: &libp2p::identity::ed25519::Keypair,
    ) -> libp2p::identity::ed25519::PublicKey {
        keypair.public()
    }
    fn secret(keypair: &libp2p::identity::ed25519::Keypair) -> Vec<u8> {
        keypair.secret().as_ref().to_vec()
    }
    fn sign(keypair: &libp2p::identity::ed25519::Keypair, msg: &[u8]) -> Vec<u8> {
        keypair.sign(msg)
    }
}

impl Verify<libp2p::identity::ed25519::PublicKey> for PublicKey {
    fn verify(public: &libp2p::identity::ed25519::PublicKey, msg: &[u8], signature: &[u8]) -> bool {
        public.verify(msg, signature)
    }

    fn decode(pk_bytes: &[u8]) -> libp2p::identity::ed25519::PublicKey{
        libp2p::identity::ed25519::PublicKey::decode(pk_bytes).unwrap()
    }
}

/// serialize a generic type T
/// Needs trait Serialize to be implemented
/// can be done directly by using macro
/// #[derive(Serialize)] defined in serde
pub fn serialize<T>(to_ser: T) -> Vec<u8>
where
    T: Serialize,
{
    let servec = serde_cbor::to_vec(&to_ser).unwrap();
    servec
}

/// returns the SHA3_256 hash of cbor value for
/// generic type T
pub fn serialize_hash256<T>(to_ser: T) -> Vec<u8>
where
    T: Serialize,
{
    let to_hash = serialize(to_ser);
    encode(Hash::SHA3256, &to_hash).unwrap().to_vec()
}

/// deserialize a vec<u8> slice and returns
/// generic type T
/// needs to implement Deserialze trait
/// with lifetime "a"
pub fn deserialize<'a, T>(slice: &'a [u8]) -> T
where
    T: Deserialize<'a>,
{
    serde_cbor::from_slice(&slice).unwrap()
}


#[derive(Debug, Serialize, Deserialize)]
struct Transaction {
    party_a: String,
    party_b: String,
    amount: u32,
    signature: Vec<u8>
}
// struct for unit testing
#[derive(Debug, Serialize, Deserialize)]
struct Mascot {
    name: String,
    species: String,
    year_of_birth: u32,
    f: Transaction,
}


fn main() {
    poa::hello_world();
    poa::transaction::main_transaction();
    Pancakes::hello_macro();
    let ferris = Mascot {
        name: "Ferris".to_owned(),
        species: "crab".to_owned(),
        year_of_birth: 2015,
        f: Transaction {
            party_a: "A".to_owned(),
            party_b: "String".to_owned(),
            amount: 3,
            signature: vec![1,2],
        },
    };
    
    // check if hash of cbor is correct
    let _hash_of_ser = serialize_hash256(&ferris);
    // assert_eq!(
    //     hash_of_ser,
    //     vec![
    //         22, 32, 243, 254, 107, 77, 41, 89, 227, 216, 65, 39, 75, 251, 101, 176, 236, 195,
    //         140, 255, 104, 236, 140, 34, 191, 18, 210, 4, 131, 108, 12, 184, 242, 73
    //     ]
    // );

    let s = "97ba6f71a5311c4986e01798d525d0da8ee5c54acbf6ef7c3fadd1e2f624442f";
    let mut secret_bytes = hex::decode(s).expect("invalid secret");
    let kp = Keypair::generate_from(secret_bytes.as_mut_slice());
    let public_key_string = hex::encode(Keypair::public(&kp).encode());
    println!("pub : {:?}", hex::encode(Keypair::public(&kp).encode()));
    println!("secrete {:?}", hex::encode(kp.secret().as_ref()));
    let ser_obj = serialize(ferris);
    let sign = Keypair::sign(&kp, &ser_obj);
    let decode_pk = hex::decode(public_key_string).unwrap();
    println!("{:?}", decode_pk);
    let public_key_decoded = PublicKey::decode(&decode_pk);
    assert_eq!(
        true,
        PublicKey::verify(&public_key_decoded, &ser_obj, sign.as_ref())
    );
}