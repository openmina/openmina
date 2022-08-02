mod common;
mod legacy;
mod new;

pub use common::*;
pub use legacy::*;
pub use new::*;

#[cfg(test)]
mod tests {
    use mina_hasher::{create_kimchi, create_legacy, Hasher};

    use super::*;

    #[test]
    fn test_hash_account() {
        let acc = AccountLegacy::create();

        let mut hasher = create_kimchi::<AccountLegacy>(());
        hasher.update(&acc);
        let out = hasher.digest();

        println!("kimchi={}", out.to_string());

        let mut hasher = create_legacy::<AccountLegacy>(());
        hasher.update(&acc);
        let out = hasher.digest();

        println!("legacy={}", out.to_string());
    }
}
