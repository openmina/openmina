use binprot_derive::{BinProtRead, BinProtWrite};
use mina_p2p_messages::v2::LedgerHash;
use serde::{Deserialize, Serialize};

pub type SnarkJobId = LedgerHashTransition;

#[derive(
    BinProtWrite, BinProtRead, Serialize, Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone,
)]
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
        write!(f, "{}->{}", &self.source, &self.target)
    }
}

impl std::str::FromStr for LedgerHashTransition {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (source, target) = s.split_once("->").ok_or(())?;
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
            "{}:{}",
            &self.first_pass_ledger, &self.second_pass_ledger
        )
    }
}

impl std::str::FromStr for LedgerHashTransitionPasses {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first_pass, second_pass) = s.split_once(":").ok_or(())?;
        Ok(Self {
            first_pass_ledger: first_pass.parse().or(Err(()))?,
            second_pass_ledger: second_pass.parse().or(Err(()))?,
        })
    }
}
