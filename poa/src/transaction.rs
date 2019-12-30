mod keypair;
use crate::transaction::keypair::CryptoKeypair;
use crate::transaction::keypair::Verify;

pub trait Txn<T, U>{
    fn generate() -> T;
    fn validate(&self) -> bool;
    fn sign(keypair: &U, party_a: &String, party_b: &String, amount: u32) -> Vec<u8>;
}

#[derive(Debug)]
pub struct Transaction {
    party_a: String,
    party_b: String,
    amount: u32,
    signature: Vec<u8>
}

impl Txn<Transaction, libp2p::identity::ed25519::Keypair> for Transaction {
    fn validate(&self) -> bool{
        let mut signing_string = String::from("");
        signing_string = signing_string + &self.party_a;
        signing_string = signing_string + &self.party_b;
        let amount_str = &self.amount.to_string();
        signing_string = signing_string + &amount_str;
        // println!("signing string {}", signing_string);

        keypair::PublicKey::verify_from_encoded_pk(&self.party_a, signing_string.as_bytes(), &self.signature.as_ref())
    }
    
    fn sign(keypair: &libp2p::identity::ed25519::Keypair, party_a: &String, party_b: &String, amount: u32) -> Vec<u8>{
        let mut signing_string = String::from("");
        signing_string = signing_string + party_a;
        signing_string = signing_string + party_b;
        let amount_str = amount.to_string();
        signing_string = signing_string + &amount_str;
        // println!("signing string {}", signing_string);
        let sign = keypair::Keypair::sign(&keypair, signing_string.as_bytes());
        sign
    }

    fn generate() -> Transaction{
        let party_a_kp = keypair::Keypair::generate();
        let party_a: String = hex::encode(party_a_kp.public().encode());
        let party_b_kp = keypair::Keypair::generate();
        let party_b: String = hex::encode(party_b_kp.public().encode());
        let sign = Transaction::sign(&party_a_kp, &party_a, &party_b, 32);
        Transaction{
            party_a,
            party_b,
            amount: 32,
            signature: sign,
        }
    }

}

pub struct TransactionPool {
    pub pool: Vec<Transaction>,
}

pub trait TxnPool {
    fn execute(&self);
}

impl TxnPool for TransactionPool {
    fn execute(&self) {
        for txn in self.pool.iter() {
            println!("{}",txn.validate());
        }
    }
}


pub fn main_transaction() {

    let transaction_pool = TransactionPool{
        pool : vec![
            Transaction::generate(),
            Transaction::generate(),
            Transaction::generate(),
            Transaction::generate(),
        ],
    };
    transaction_pool.execute();
}