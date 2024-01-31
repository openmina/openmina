use ark_ff::{PrimeField, ToBytes};
use mina_hasher::Fp;
use sha2::{Digest, Sha256};

use crate::{proofs::field::FieldWitness, scan_state::pending_coinbase::PendingCoinbase, ToInputs};

/// Convert to/from OCaml strings, such as
/// "u~\218kzX\228$\027qG\239\135\255:\143\171\186\011\200P\243\163\135\223T>\017\172\254\1906"
pub trait OCamlString {
    fn to_ocaml_str(&self) -> String;
    fn from_ocaml_str(s: &str) -> Self;
}

impl<const N: usize> OCamlString for [u8; N] {
    fn to_ocaml_str(&self) -> String {
        to_ocaml_str(self)
    }

    fn from_ocaml_str(s: &str) -> Self {
        from_ocaml_str(s)
    }
}

pub fn to_ocaml_str(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(256);

    for b in bytes {
        let c = char::from(*b);
        if c == '\\' {
            s.push_str("\\\\");
        } else if c.is_ascii() && !c.is_ascii_control() {
            s.push(c);
        } else {
            match b {
                7 => s.push_str(r"\a"),
                8 => s.push_str(r"\b"),
                9 => s.push_str(r"\t"),
                10 => s.push_str(r"\n"),
                // 11 => s.push_str(r"\v"),
                12 => s.push_str(r"\f"),
                13 => s.push_str(r"\r"),
                _ => s.push_str(&format!("\\{:<03}", b)),
            }
        }
    }

    s
}

pub fn from_ocaml_str<const N: usize>(s: &str) -> [u8; N] {
    let mut bytes = [0; N];
    let mut b_index = 0;

    let mut index = 0;
    let s = s.as_bytes();
    while index < s.len() {
        if s[index] == b'\\' {
            if s.get(index + 1).map(|next| *next == b'\\').unwrap_or(false) {
                bytes[b_index] = b'\\';
                index += 2;
            } else if s
                .get(index + 1)
                .map(|next| "abtnfr".contains(char::from(*next)))
                // .map(|next| "abtnvfr".contains(char::from(*next)))
                .unwrap_or(false)
            {
                bytes[b_index] = match s[index + 1] {
                    b'a' => 7,
                    b'b' => 8,
                    b't' => 9,
                    b'n' => 10,
                    // b'v' => 11,
                    b'f' => 12,
                    b'r' => 13,
                    _ => unreachable!(),
                };
                index += 2;
            } else {
                let n1 = s[index + 1] - b'0';
                let n2 = s[index + 2] - b'0';
                let n3 = s[index + 3] - b'0';
                bytes[b_index] = (n1 * 100) + (n2 * 10) + n3;
                index += 4;
            }
        } else {
            bytes[b_index] = s[index];
            index += 1;
        }

        b_index += 1;
    }

    bytes
}

/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L27
#[derive(PartialEq, Eq)]
pub struct AuxHash(pub [u8; 32]);

impl std::fmt::Debug for AuxHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("AuxHash({})", self.to_ocaml_str()))
    }
}

impl AuxHash {
    fn to_ocaml_str(&self) -> String {
        to_ocaml_str(&self.0)
    }

    fn from_ocaml_str(s: &str) -> Self {
        Self(from_ocaml_str(s))
    }
}

/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L110
#[derive(PartialEq, Eq)]
pub struct PendingCoinbaseAux(pub [u8; 32]);

impl std::fmt::Debug for PendingCoinbaseAux {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("PendingCoinbaseAux({})", self.to_ocaml_str()))
    }
}

impl PendingCoinbaseAux {
    fn to_ocaml_str(&self) -> String {
        to_ocaml_str(&self.0)
    }

    fn from_ocaml_str(s: &str) -> Self {
        Self(from_ocaml_str(s))
    }
}

/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L152
#[derive(Debug, PartialEq, Eq)]
pub struct NonStark {
    pub ledger_hash: Fp,
    pub aux_hash: AuxHash,
    pub pending_coinbase_aux: PendingCoinbaseAux,
}

impl NonStark {
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L182
    pub fn digest(&self) -> [u8; 32] {
        let Self {
            ledger_hash,
            aux_hash,
            pending_coinbase_aux,
        } = self;

        let mut sha: Sha256 = Sha256::new();

        let mut ledger_hash_bytes: [u8; 32] = <[u8; 32]>::default();

        let ledger_hash = ledger_hash.into_repr();
        ledger_hash.write(ledger_hash_bytes.as_mut_slice()).unwrap();
        ledger_hash_bytes.reverse();

        sha.update(ledger_hash_bytes.as_slice());
        sha.update(aux_hash.0.as_slice());
        sha.update(pending_coinbase_aux.0.as_slice());

        sha.finalize().into()
    }
}

impl ToInputs for NonStark {
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L193
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        let digest = self.digest();
        inputs.append_bytes(digest.as_slice());
    }
}

/// Staged ledger hash has two parts
///
/// 1) merkle root of the pending coinbases
/// 2) ledger hash, aux hash, and the FIFO order of the coinbase stacks(Non snark).
///
/// Only part 1 is required for blockchain snark computation and therefore the
/// remaining fields of the staged ledger are grouped together as "Non_snark"
///
/// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L259
#[derive(Debug, PartialEq, Eq)]
pub struct StagedLedgerHash<F: FieldWitness> {
    pub non_snark: NonStark,
    pub pending_coinbase_hash: F,
}

impl StagedLedgerHash<Fp> {
    /// https://github.com/MinaProtocol/mina/blob/3fe924c80a4d01f418b69f27398f5f93eb652514/src/lib/mina_base/staged_ledger_hash.ml#L290
    pub fn of_aux_ledger_and_coinbase_hash(
        aux_hash: AuxHash,
        ledger_hash: Fp,
        pending_coinbase: &mut PendingCoinbase,
    ) -> Self {
        Self {
            non_snark: NonStark {
                ledger_hash,
                aux_hash,
                pending_coinbase_aux: pending_coinbase.hash_extra(),
            },
            pending_coinbase_hash: pending_coinbase.merkle_root(),
        }
    }

    /// Used for tests only
    #[cfg(test)]
    pub fn from_ocaml_strings(
        ledger_hash: &str,
        aux_hash: &str,
        pending_coinbase_aux: &str,
        pending_coinbase_hash: &str,
    ) -> Self {
        use std::str::FromStr;

        Self {
            non_snark: NonStark {
                ledger_hash: Fp::from_str(ledger_hash).unwrap(),
                aux_hash: AuxHash::from_ocaml_str(aux_hash),
                pending_coinbase_aux: PendingCoinbaseAux::from_ocaml_str(pending_coinbase_aux),
            },
            pending_coinbase_hash: Fp::from_str(pending_coinbase_hash).unwrap(),
        }
    }
}

impl ToInputs for StagedLedgerHash<Fp> {
    fn to_inputs(&self, inputs: &mut crate::Inputs) {
        let Self {
            non_snark,
            pending_coinbase_hash,
        } = self;

        inputs.append(non_snark);
        inputs.append(pending_coinbase_hash);
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    #[cfg(target_family = "wasm")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use super::*;

    #[test]
    fn test_convert() {
        // stage_ledger_hash=StagedLedgerHash {
        //     non_snark: NonStark {
        //         ledger_hash: Fp(7213023165825031994332898585791275635753820608093286100176380057570051468967),
        //         aux_hash: AuxHash(\\\249\245k\176]TJ\216\183\001\204\177\131\030\244o\178\188\191US\156\192Hi\194P\223\004\000\003),
        //         pending_coinbase_aux: PendingCoinbaseAux(\\\236\235f\255\200o8\217Rxlmily\194\219\1949\221N\145\180g)\215:'\251W\233),
        //     },
        //     pending_coinbase_hash: Fp(25504365445533103805898245102289650498571312278321176071043666991586378788150),
        // }

        let s = r"\\\236\235f\255\200o8\217Rxlmily\194\219\1949\221N\145\180g)\215:'\251W\233";
        let pending_coinbase_aux = PendingCoinbaseAux::from_ocaml_str(s);
        assert_eq!(s, pending_coinbase_aux.to_ocaml_str());

        let s = r"\\\249\245k\176]TJ\216\183\001\204\177\131\030\244o\178\188\191US\156\192Hi\194P\223\004\000\003";
        let aux_hash = AuxHash::from_ocaml_str(s);
        assert_eq!(s, aux_hash.to_ocaml_str());

        // non_snark digest="\t\204S\160F\227\022\142\146\172\220.R'\222L&b\191\138;\022\235\137\190>\205.\031\195-\231"
        // digest=9,204,83,160,70,227,22,142,146,172,220,46,82,39,222,76,38,98,191,138,59,22,235,137,190,62,205,46,31,195,45,231

        let a = AuxHash([
            9, 204, 83, 160, 70, 227, 22, 142, 146, 172, 220, 46, 82, 39, 222, 76, 38, 98, 191,
            138, 59, 22, 235, 137, 190, 62, 205, 46, 31, 195, 45, 231,
        ]);

        println!("a={}", a.to_ocaml_str());

        let s = r"\t\204S\160F\227\022\142\146\172\220.R'\222L&b\191\138;\022\235\137\190>\205.\031\195-\231";
        assert_eq!(s, a.to_ocaml_str());
        let aux_hash = AuxHash::from_ocaml_str(s);
        assert_eq!(s, aux_hash.to_ocaml_str());

        let s = r"\000 \014WQ\192&\229C\178\232\171.\176`\153\218\161\209\229\223Gw\143w\135\250\171E\205\241/\227\168";
        let memo = <[u8; 34]>::from_ocaml_str(s);
        assert_eq!(s, memo.to_ocaml_str());

        // let bytes = [10,220,211,153,14,65,191,6,19,231,47,244,155,5,212,131,48,124,227,133,176,79,196,131,23,116,152,178,130,63,206,85];

        // let s = bytes.to_ocaml_str();
        // println!("s='{}'", s);

        let s = r"\n\220\211\153\014A\191\006\019\231/\244\155\005\212\1310|\227\133\176O\196\131\023t\152\178\130?\206U";
        let pending_coinbase_aux = PendingCoinbaseAux::from_ocaml_str(s);
        assert_eq!(s, pending_coinbase_aux.to_ocaml_str());
    }

    #[test]
    fn test_non_snark_digest() {
        // ((non_snark
        //   ((ledger_hash
        //     7213023165825031994332898585791275635753820608093286100176380057570051468967)
        //    (aux_hash
        //     "T\249\245k\176]TJ\216\183\001\204\177\131\030\244o\178\188\191US\156\192Hi\194P\223\004\000\003")
        //    (pending_coinbase_aux
        //     "_\236\235f\255\200o8\217Rxlmily\194\219\1949\221N\145\180g)\215:'\251W\233")))

        let non_snark = NonStark {
            ledger_hash: Fp::from_str(
                "7213023165825031994332898585791275635753820608093286100176380057570051468967",
            )
            .unwrap(),
            aux_hash: AuxHash::from_ocaml_str(
                r"T\249\245k\176]TJ\216\183\001\204\177\131\030\244o\178\188\191US\156\192Hi\194P\223\004\000\003",
            ),
            pending_coinbase_aux: PendingCoinbaseAux::from_ocaml_str(
                r"_\236\235f\255\200o8\217Rxlmily\194\219\1949\221N\145\180g)\215:'\251W\233",
            ),
        };

        assert_eq!(
            non_snark.digest().to_ocaml_str(),
            r"\t\204S\160F\227\022\142\146\172\220.R'\222L&b\191\138;\022\235\137\190>\205.\031\195-\231"
        );

        // non_snark=((ledger_hash
        //   18582860218764414485081234471609377222894570081548691702645303871998665679024)
        //  (aux_hash
        //   "0\136Wg\182DbX\203kLi\212%\199\206\142#\213`L\160bpCB\1413\240\193\171K")
        //  (pending_coinbase_aux
        //    "\
        //   \n\220\211\153\014A\191\006\019\231/\244\155\005\212\1310|\227\133\176O\196\131\023t\152\178\130?\206U"))

        let non_snark = NonStark {
            ledger_hash: Fp::from_str(
                "18582860218764414485081234471609377222894570081548691702645303871998665679024",
            )
            .unwrap(),
            aux_hash: AuxHash::from_ocaml_str(
                r"0\136Wg\182DbX\203kLi\212%\199\206\142#\213`L\160bpCB\1413\240\193\171K",
            ),
            pending_coinbase_aux: PendingCoinbaseAux::from_ocaml_str(
                r"\n\220\211\153\014A\191\006\019\231/\244\155\005\212\1310|\227\133\176O\196\131\023t\152\178\130?\206U",
            ),
        };

        assert_eq!(
            non_snark.digest().to_ocaml_str(),
            r"u~\218kzX\228$\027qG\239\135\255:\143\171\186\011\200P\243\163\135\223T>\017\172\254\1906",
        );
    }
}
