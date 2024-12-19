use serde::{Deserialize, Serialize};

use super::{Transaction, TransactionHash};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionWithHash<T: AsRef<Transaction> = Transaction> {
    hash: TransactionHash,
    body: T,
}

impl<T: AsRef<Transaction>> TransactionWithHash<T> {
    pub fn try_new(body: T) -> std::io::Result<Self> {
        Ok(Self {
            hash: body.as_ref().hash()?,
            body,
        })
    }

    pub fn hash(&self) -> &TransactionHash {
        &self.hash
    }

    pub fn body(&self) -> &Transaction {
        self.body.as_ref()
    }

    pub fn into_body(self) -> T {
        self.body
    }
}
