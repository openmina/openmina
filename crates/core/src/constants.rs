use binprot_derive::BinProtWrite;
use mina_curves::pasta::Fp;
use mina_p2p_messages::{bigint, number, v2};

pub const GENESIS_PRODUCER_SK: &str = "EKFKgDtU3rcuFTVSEpmpXSkukjmX4cKefYREi6Sdsk7E7wsT7KRw";

pub const PROTOCOL_VERSION: v2::ProtocolVersionStableV2 = v2::ProtocolVersionStableV2 {
    transaction: number::Number(3),
    network: number::Number(0),
    patch: number::Number(0),
};

pub fn constraint_constants() -> &'static ConstraintConstants {
    NetworkConfig::global().constraint_constants
}

/// Constants that define fork-specific blockchain state.
///
/// Fork constants specify the blockchain state at which a protocol upgrade or fork occurred.
/// These are used to handle protocol changes and ensure compatibility across network upgrades.
#[derive(Clone, Debug)]
pub struct ForkConstants {
    /// Hash of the blockchain state at the fork point
    pub state_hash: Fp,

    /// Blockchain length (number of blocks) at the fork point
    pub blockchain_length: u32,

    /// Global slot number since genesis at the fork point
    pub global_slot_since_genesis: u32,
}

/// Protocol constraint constants that define core blockchain behavior.
///
/// These constants configure fundamental aspects of the Mina protocol including consensus,
/// transaction processing, economic parameters, and ledger structure. They are compile-time
/// parameters that must be consistent across all nodes in a network.
///
/// ## Consensus and Timing Parameters
///
/// The consensus mechanism relies on slot-based timing where blocks are
/// produced in discrete time slots. The timing hierarchy is:
/// - **Slots**: Basic time units for block production
/// - **Sub-windows**: Groups of slots within an epoch
/// - **Windows**: Collections of sub-windows that define epoch structure
/// - **Epochs**: Complete consensus periods
///
/// ## Economic Parameters
///
/// The protocol defines economic incentives through fees and rewards:
/// - **Coinbase rewards**: Paid to block producers for successful blocks
/// - **Account creation fees**: Required to create new accounts on the ledger
/// - **Supercharged rewards**: Multiplier for enhanced block producer rewards
///
/// ## Ledger and Transaction Structure
///
/// The ledger uses a Merkle tree structure for efficient verification:
/// - **Ledger depth**: Determines the maximum number of accounts (2^depth)
/// - **Transaction capacity**: Limits transactions per block for performance
/// - **Pending coinbase**: Manages delayed coinbase payouts
///
/// ## Usage Example
///
/// ```rust
/// use mina_core::constants::constraint_constants;
///
/// // Access global constraint constants
/// let constants = constraint_constants();
///
/// // Calculate slots per window
/// let slots_per_window = constants.sub_windows_per_window;
/// println!("Sub-windows per window: {}", slots_per_window);
///
/// // Get block timing
/// let block_time_ms = constants.block_window_duration_ms;
/// println!("Block time: {}ms", block_time_ms);
///
/// // Check economic parameters
/// let coinbase_reward = constants.coinbase_amount;
/// let creation_fee = constants.account_creation_fee;
/// println!("Coinbase: {} nanomina, Account fee: {} nanomina",
///          coinbase_reward, creation_fee);
/// ```
///
/// ## Network Differences
///
/// While most constraint constants are identical across networks, some parameters
/// may differ between mainnet and testnets for development purposes.
///
/// Related OCaml implementation: <https://github.com/MinaProtocol/mina/tree/compatible/src/config>
/// Protocol specification: <https://github.com/MinaProtocol/mina/blob/compatible/docs/specs/types_and_structures/serialized_key.md>
#[derive(Clone, Debug)]
pub struct ConstraintConstants {
    /// Number of sub-windows that make up a complete window.
    ///
    /// Used in the consensus mechanism to structure epoch timing. Combined with
    /// `slots_per_sub_window` from protocol constants, this determines the total
    /// slots per window: `slots_per_window = slots_per_sub_window × sub_windows_per_window`.
    ///
    /// **Value**: 11 (both mainnet and devnet)
    pub sub_windows_per_window: u64,

    /// Depth of the account ledger Merkle tree.
    ///
    /// This determines the maximum number of accounts that can be stored in the ledger:
    /// `max_accounts = 2^ledger_depth`. The depth affects proof sizes and verification time.
    /// A larger depth allows more accounts but increases computational overhead.
    ///
    /// **Value**: 35 (supports ~34 billion accounts)
    /// **Usage**: Account addressing, sparse ledger proofs, zkSNARK constraints
    pub ledger_depth: u64,

    /// Number of blocks to delay before SNARK work becomes available.
    ///
    /// This creates a buffer period between when a block is produced and when
    /// the associated SNARK work can be included in subsequent blocks. This delay
    /// helps ensure fair distribution of SNARK work opportunities.
    ///
    /// **Value**: 2 blocks
    /// **Usage**: SNARK work scheduling, proof marketplace timing
    pub work_delay: u64,

    /// Duration of each block production slot in milliseconds.
    ///
    /// This is the fundamental time unit for the consensus protocol. Block producers
    /// attempt to create blocks during their assigned slots. The duration affects
    /// network synchronization requirements and transaction confirmation times.
    ///
    /// **Value**: 180,000ms (3 minutes)
    /// **Usage**: Consensus timing, slot calculations, network synchronization
    pub block_window_duration_ms: u64,

    /// Log₂ of the maximum number of transactions per block.
    ///
    /// The actual transaction capacity is `2^transaction_capacity_log_2`. This logarithmic
    /// representation is used because the value directly affects zkSNARK circuit constraints.
    /// Higher capacity allows more transactions but increases block processing time.
    ///
    /// Corresponds to `transaction_capacity` in the protocol specification, which defines
    /// the maximum transactions per block (represented as `two_to_the`).
    ///
    /// **Value**: 7 (supports 2^7 = 128 transactions per block)
    /// **Usage**: Transaction pool management, block construction, circuit constraints
    pub transaction_capacity_log_2: u64,

    /// Number of confirmations before coinbase reward is spendable.
    ///
    /// Coinbase rewards are not immediately spendable and require a certain number
    /// of block confirmations before they can be used. This parameter defines the
    /// depth of the pending coinbase Merkle tree structure used to track these
    /// delayed rewards until they mature.
    ///
    /// **Value**: 5 (coinbase rewards require 5 block confirmations)
    /// **Usage**: Coinbase reward management, staged ledger operations, reward maturity
    pub pending_coinbase_depth: usize,

    /// Block reward amount in nanomina (10⁻⁹ MINA).
    ///
    /// This is the base reward paid to block producers for successfully creating a block.
    /// The amount is specified in nanomina, where 1 MINA = 10⁹ nanomina. Block producers
    /// may receive additional rewards through the supercharged coinbase mechanism.
    ///
    /// **Value**: 720,000,000,000 nanomina (720 MINA)
    /// **Usage**: Block producer rewards, economic incentives, reward calculations
    pub coinbase_amount: u64,

    /// Multiplier for supercharged coinbase rewards.
    ///
    /// Supercharged rewards were designed to provide double block rewards (factor of 2)
    /// to block producers staking with unlocked tokens during the early mainnet period
    /// following the 2021 launch. This mechanism incentivized participation and orderly
    /// markets after mainnet launch.
    ///
    /// **Historical values**:
    /// - Original mainnet: 2 (double rewards for unlocked tokens)
    /// - Berkeley hardfork (June 2024): 1 (supercharged rewards removed via MIP1)
    ///
    /// The removal was decided by community vote on January 1, 2023, as proposed by
    /// community member Gareth Davies. This change ensures uniform rewards for all
    /// tokens and reduces inflation, promoting a sustainable economic model.
    ///
    /// **References**:
    /// - Berkeley Upgrade: <https://minaprotocol.com/blog/minas-berkeley-upgrade-what-to-expect>
    /// - Supercharged Rewards Removal: <https://minaprotocol.com/blog/update-on-minas-supercharged-rewards-schedule>
    /// - Original Proposal: <https://github.com/MinaProtocol/mina/issues/5753>
    ///
    /// **Usage**: Enhanced reward calculations, incentive mechanisms
    pub supercharged_coinbase_factor: u64,

    /// Fee required to create a new account in nanomina.
    ///
    /// When a transaction creates a new account that doesn't exist on the ledger,
    /// this fee is charged in addition to the transaction fee. This prevents
    /// spam account creation and manages ledger growth.
    ///
    /// **Value**: 1,000,000,000 nanomina (1 MINA)
    /// **Usage**: Account creation, transaction validation, fee calculations
    pub account_creation_fee: u64,

    /// Optional fork constants defining a protocol upgrade point.
    ///
    /// When present, these constants specify the blockchain state at which a protocol
    /// fork or upgrade occurred. This allows the protocol to handle transitions between
    /// different versions while maintaining consensus.
    ///
    /// **Usage**: Protocol upgrades, compatibility handling, genesis configuration
    pub fork: Option<ForkConstants>,
}
#[derive(Clone, Debug, BinProtWrite)]
pub struct ForkConstantsUnversioned {
    previous_state_hash: bigint::BigInt,
    previous_length: number::Int32,
    genesis_slot: number::Int32,
}

impl From<&ForkConstants> for ForkConstantsUnversioned {
    fn from(fork_constants: &ForkConstants) -> Self {
        Self {
            previous_state_hash: fork_constants.state_hash.into(),
            previous_length: fork_constants.blockchain_length.into(),
            genesis_slot: fork_constants.global_slot_since_genesis.into(),
        }
    }
}

#[derive(Clone, Debug, BinProtWrite)]
pub struct ConstraintConstantsUnversioned {
    pub sub_windows_per_window: number::Int64,
    pub ledger_depth: number::Int64,
    pub work_delay: number::Int64,
    pub block_window_duration_ms: number::Int64,
    pub transaction_capacity_log_2: number::Int64,
    pub pending_coinbase_depth: number::Int64,
    pub coinbase_amount: number::UInt64,
    pub supercharged_coinbase_factor: number::Int64,
    pub account_creation_fee: number::UInt64,
    pub fork: Option<ForkConstantsUnversioned>,
}

impl From<&ConstraintConstants> for ConstraintConstantsUnversioned {
    fn from(constraints: &ConstraintConstants) -> Self {
        Self {
            sub_windows_per_window: constraints.sub_windows_per_window.into(),
            ledger_depth: constraints.ledger_depth.into(),
            work_delay: constraints.work_delay.into(),
            block_window_duration_ms: constraints.block_window_duration_ms.into(),
            transaction_capacity_log_2: constraints.transaction_capacity_log_2.into(),
            pending_coinbase_depth: (constraints.pending_coinbase_depth as u64).into(),
            coinbase_amount: constraints.coinbase_amount.into(),
            supercharged_coinbase_factor: constraints.supercharged_coinbase_factor.into(),
            account_creation_fee: constraints.account_creation_fee.into(),
            fork: constraints.fork.as_ref().map(|fork| fork.into()),
        }
    }
}

impl binprot::BinProtWrite for ConstraintConstants {
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let constraints: ConstraintConstantsUnversioned = self.into();
        constraints.binprot_write(w)
    }
}

pub fn slots_per_window(constants: &v2::MinaBaseProtocolConstantsCheckedValueStableV1) -> u32 {
    constants.slots_per_sub_window.as_u32() * (constraint_constants().sub_windows_per_window as u32)
}

const fn days_to_ms(days: u64) -> u64 {
    days * 24 * 60 * 60 * 1000
}

pub const CHECKPOINTS_PER_YEAR: u64 = 12;

pub fn checkpoint_window_size_in_slots() -> u32 {
    let one_year_ms = days_to_ms(365);
    let slots_per_year = one_year_ms / constraint_constants().block_window_duration_ms;
    let size_in_slots = slots_per_year / CHECKPOINTS_PER_YEAR;
    assert_eq!(slots_per_year % CHECKPOINTS_PER_YEAR, 0);
    size_in_slots as u32
}

pub const DEFAULT_GENESIS_TIMESTAMP_MILLISECONDS: u64 = 1707157200000;

pub const PROTOCOL_TRANSACTION_VERSION: u8 = 3;
pub const PROTOCOL_NETWORK_VERSION: u8 = 3;
pub const TX_POOL_MAX_SIZE: u32 = 3000;

pub use v2::PROTOCOL_CONSTANTS;

use crate::NetworkConfig;
