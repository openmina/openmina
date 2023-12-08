use std::str::FromStr;

use ledger::scan_state::scan_state::{
    transaction_snark::{OneOrTwo, Statement},
    AvailableJobMessage,
};
use mina_p2p_messages::binprot::{
    self,
    macros::{BinProtRead, BinProtWrite},
};
use mina_p2p_messages::v2::{
    LedgerHash, MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    TransactionSnarkWorkTStableV2Proofs,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize};

pub type SnarkJobId = LedgerHashTransition;

#[derive(BinProtWrite, BinProtRead, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct LedgerHashTransition {
    pub source: LedgerHashTransitionPasses,
    pub target: LedgerHashTransitionPasses,
}

#[derive(
    BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone,
)]
pub struct LedgerHashTransitionPasses {
    pub first_pass_ledger: LedgerHash,
    pub second_pass_ledger: LedgerHash,
}

impl std::fmt::Display for LedgerHashTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", &self.source, &self.target)
    }
}

impl std::str::FromStr for LedgerHashTransition {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (source, target) = s.split_once("-").ok_or(())?;
        Ok(Self {
            source: source.parse()?,
            target: target.parse()?,
        })
    }
}

impl std::fmt::Display for LedgerHashTransitionPasses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}",
            &self.first_pass_ledger, &self.second_pass_ledger
        )
    }
}

impl FromStr for LedgerHashTransitionPasses {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first_pass, second_pass) = s.split_once("_").ok_or(())?;
        Ok(Self {
            first_pass_ledger: first_pass.parse().or(Err(()))?,
            second_pass_ledger: second_pass.parse().or(Err(()))?,
        })
    }
}

impl Serialize for LedgerHashTransition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            let mut s = serializer.serialize_struct("LedgerHashTransition", 2)?;
            s.serialize_field("source", &self.source)?;
            s.serialize_field("target", &self.target)?;
            s.end()
        }
    }
}

impl<'de> Deserialize<'de> for LedgerHashTransition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        if deserializer.is_human_readable() {
            let s: String = Deserialize::deserialize(deserializer)?;
            Self::from_str(&s).map_err(|_| Error::custom("decode from str failed"))
        } else {
            #[derive(Deserialize)]
            struct LedgerHashTransition {
                pub source: LedgerHashTransitionPasses,
                pub target: LedgerHashTransitionPasses,
            }
            let v: LedgerHashTransition = Deserialize::deserialize(deserializer)?;
            Ok(Self {
                source: v.source,
                target: v.target,
            })
        }
    }
}

impl From<&OneOrTwo<AvailableJobMessage>> for SnarkJobId {
    fn from(value: &OneOrTwo<AvailableJobMessage>) -> Self {
        let (first, second) = match value {
            OneOrTwo::One(j) => (j, j),
            OneOrTwo::Two((j1, j2)) => (j1, j2),
        };

        let source = match first {
            AvailableJobMessage::Base(base) => &base.statement.0.source,
            AvailableJobMessage::Merge { left, .. } => &left.0 .0.statement.source,
        };
        let target = match second {
            AvailableJobMessage::Base(base) => &base.statement.0.target,
            AvailableJobMessage::Merge { right, .. } => &right.0 .0.statement.target,
        };

        (source, target).into()
    }
}

impl From<&OneOrTwo<Statement<()>>> for SnarkJobId {
    fn from(value: &OneOrTwo<Statement<()>>) -> Self {
        let (source, target): (
            MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
            MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
        ) = match value {
            OneOrTwo::One(stmt) => ((&stmt.source).into(), (&stmt.target).into()),
            OneOrTwo::Two((stmt1, stmt2)) => ((&stmt1.source).into(), (&stmt2.target).into()),
        };
        (&source, &target).into()
    }
}

impl From<&TransactionSnarkWorkTStableV2Proofs> for SnarkJobId {
    fn from(value: &TransactionSnarkWorkTStableV2Proofs) -> Self {
        let (first, second) = match value {
            TransactionSnarkWorkTStableV2Proofs::One(j) => (j, j),
            TransactionSnarkWorkTStableV2Proofs::Two((j1, j2)) => (j1, j2),
        };

        (&first.0.statement.source, &second.0.statement.target).into()
    }
}

impl
    From<(
        &MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
        &MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
    )> for SnarkJobId
{
    fn from(
        (source, target): (
            &MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
            &MinaStateBlockchainStateValueStableV2LedgerProofStatementSource,
        ),
    ) -> Self {
        let source = LedgerHashTransitionPasses {
            first_pass_ledger: source.first_pass_ledger.clone(),
            second_pass_ledger: source.second_pass_ledger.clone(),
        };
        let target = LedgerHashTransitionPasses {
            first_pass_ledger: target.first_pass_ledger.clone(),
            second_pass_ledger: target.second_pass_ledger.clone(),
        };

        LedgerHashTransition { source, target }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snark_job_id_to_string_from_string() {
        let s = "jw9nPCs68UNaKaLZwV6QzdswKWomwQxvTgrpmKWmnFJyswnrn4N:jwhHYWzvJG8esmqtYXbUZy3UGbLSjhKvn1FSxBGL1JDFHqbHMJc->jwiLuRrEqNgASgXEqibGs4VqKwSwiuFEtuPD53v8hiTtVuLfmTr:jwhHYWzvJG8esmqtYXbUZy3UGbLSjhKvn1FSxBGL1JDFHqbHMJc";
        let decoded = SnarkJobId::from_str(s).unwrap();
        assert_eq!(decoded.to_string(), s);
    }
}
