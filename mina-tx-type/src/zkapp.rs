//! zkApp command types
//!
//! This module defines the data structures for zkApp commands, which are
//! complex multi-account transactions that use zero-knowledge proofs for
//! authorization.
//!
//! # Structure Overview
//!
//! A [`ZkAppCommand`] contains:
//! - [`FeePayer`]: The account paying transaction fees
//! - [`CallForest`]: A tree of account updates
//! - [`Memo`]: User-defined transaction metadata
//!
//! # Authorization
//!
//! zkApp commands support three authorization methods:
//! - **Proof**: Verified against the account's verification key
//! - **Signature**: Signed by the account's private key
//! - **None**: No authorization (for certain permitted operations)
//!
//! # Preconditions
//!
//! Account updates can specify preconditions that must be satisfied:
//! - Network state (blockchain length, global slot, etc.)
//! - Account state (balance, nonce, app state, etc.)
//! - Time validity windows

extern crate alloc;

use alloc::vec::Vec;

/// A zkApp command representing a complex multi-account transaction.
///
/// zkApp commands enable sophisticated transaction logic through zero-knowledge
/// proofs. Unlike signed commands, they can update multiple accounts atomically
/// and enforce complex preconditions.
///
/// # Fields
///
/// - `fee_payer`: The account responsible for paying the transaction fee. This
///   account must authorize the fee payment with a signature.
/// - `account_updates`: A forest (tree) of account updates to apply. Updates
///   are processed in a specific order and can have parent-child relationships.
/// - `memo`: Optional user-defined metadata (up to 32 bytes of user data).
///
/// # Example Structure
///
/// ```text
/// ZkAppCommand
/// +-- fee_payer: FeePayer (pays fees, always signed)
/// +-- account_updates: CallForest
/// |   +-- AccountUpdate (can be proof/signature/none authorized)
/// |   |   +-- children: CallForest (nested updates)
/// |   +-- AccountUpdate
/// |       +-- children: CallForest
/// +-- memo: "user memo"
/// ```
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_command.ml`
#[derive(Debug, Clone, PartialEq)]
pub struct ZkAppCommand<Pk, Fp, Auth> {
    /// The account paying the transaction fee.
    ///
    /// The fee payer is always authorized by a signature and pays for all
    /// computation and storage costs of the transaction. The fee payer's
    /// nonce is incremented to prevent replay attacks.
    pub fee_payer: FeePayer<Pk>,

    /// A tree of account updates to apply atomically.
    ///
    /// Account updates are organized in a forest structure where updates can
    /// have child updates. This enables complex transaction patterns like
    /// token transfers that require updates to multiple accounts.
    pub account_updates: CallForest<AccountUpdate<Pk, Fp, Auth>>,

    /// User-defined transaction memo.
    ///
    /// A 34-byte field where the first 2 bytes encode the length and format,
    /// and up to 32 bytes contain user data. Commonly used for transaction
    /// descriptions or external reference IDs.
    pub memo: Memo,
}

/// The fee payer for a zkApp command.
///
/// The fee payer is a special account update that:
/// - Always uses the default token (MINA)
/// - Must be authorized by a signature
/// - Pays the transaction fee
/// - Has its nonce incremented
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (FeePayer module)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeePayer<Pk> {
    /// The fee payer's account details and fee information.
    pub body: FeePayerBody<Pk>,

    /// The signature authorizing the fee payment.
    ///
    /// This signature covers the entire zkApp command, including all account
    /// updates, ensuring the fee payer consents to the full transaction.
    pub authorization: Signature,
}

/// The body of a fee payer, containing account and fee details.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (FeePayerBody)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeePayerBody<Pk> {
    /// The public key of the fee payer account.
    ///
    /// This identifies which account will pay the transaction fee and have
    /// its nonce incremented.
    pub public_key: Pk,

    /// The fee to pay for the transaction.
    ///
    /// The fee compensates block producers for including the transaction and
    /// must be sufficient for the transaction's computational cost.
    pub fee: Fee,

    /// Optional slot until which the transaction is valid.
    ///
    /// If `Some(slot)`, the transaction will fail if applied after this slot.
    /// If `None`, the transaction has no expiration.
    pub valid_until: Option<Slot>,

    /// The fee payer's account nonce.
    ///
    /// Must match the current nonce of the fee payer's account. The nonce is
    /// incremented after the transaction is applied to prevent replay attacks.
    pub nonce: Nonce,
}

/// A forest of account updates with cryptographic commitments.
///
/// The call forest represents a tree structure where each account update can
/// have child updates. This enables complex transaction patterns where one
/// account's update depends on or triggers updates to other accounts.
///
/// # Structure
///
/// Each node in the forest contains:
/// - An account update
/// - A cryptographic hash for efficient commitment
/// - Potentially nested child updates
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_command.ml` (CallForest)
#[derive(Debug, Clone, PartialEq)]
pub struct CallForest<AccUpdate>(pub Vec<WithStackHash<AccUpdate>>);

/// An account update wrapped with its stack hash for cryptographic commitment.
///
/// The stack hash enables efficient verification of the account update tree
/// without examining every node.
#[derive(Debug, Clone, PartialEq)]
pub struct WithStackHash<AccUpdate> {
    /// The account update and its children.
    pub elt: Tree<AccUpdate>,

    /// The cryptographic hash of this subtree.
    ///
    /// This hash commits to this account update and all its descendants,
    /// enabling efficient Merkle-style proofs.
    pub stack_hash: StackHash,
}

/// A tree node containing an account update and its children.
#[derive(Debug, Clone, PartialEq)]
pub struct Tree<AccUpdate> {
    /// The account update at this node.
    pub account_update: AccUpdate,

    /// The stack hash of just this account update (without children).
    pub account_update_digest: AccountUpdateDigest,

    /// Child account updates that depend on this update.
    pub calls: CallForest<AccUpdate>,
}

/// An account update specifying changes to an account's state.
///
/// Account updates are the core building blocks of zkApp commands. Each update
/// specifies:
/// - Which account to modify (by public key and token)
/// - What changes to make (balance, app state, permissions, etc.)
/// - What preconditions must be satisfied
/// - How the update is authorized
///
/// # Authorization
///
/// Updates can be authorized in three ways:
/// - **Proof**: A zero-knowledge proof verified against the account's
///   verification key
/// - **Signature**: A cryptographic signature from the account's private key
/// - **None**: No authorization (only valid for certain operations)
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml`
#[derive(Debug, Clone, PartialEq)]
pub struct AccountUpdate<Pk, Fp, Auth> {
    /// The body containing all update details and preconditions.
    pub body: AccountUpdateBody<Pk, Fp>,

    /// The authorization for this update.
    ///
    /// Must match the `authorization_kind` specified in the body.
    pub authorization: Auth,
}

/// The body of an account update containing all modification details.
///
/// This structure specifies exactly what changes should be made to an account
/// and under what conditions those changes are permitted.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (Body)
#[derive(Debug, Clone, PartialEq)]
pub struct AccountUpdateBody<Pk, Fp> {
    /// The public key of the account to update.
    pub public_key: Pk,

    /// The token ID for this account.
    ///
    /// Combined with the public key, this uniquely identifies an account.
    /// The default token ID represents MINA.
    pub token_id: TokenId<Fp>,

    /// The updates to apply to the account's state.
    ///
    /// Each field can be `Set` to a new value or `Keep` the existing value.
    pub update: Update<Pk, Fp>,

    /// The change to the account's balance.
    ///
    /// Positive values increase the balance (receiving), negative values
    /// decrease it (sending). The magnitude and sign are specified separately.
    pub balance_change: Signed<Amount>,

    /// Whether to increment the account's nonce.
    ///
    /// Should be `true` for updates that should only be applied once,
    /// preventing replay attacks.
    pub increment_nonce: bool,

    /// Events emitted by this account update.
    ///
    /// Events are arbitrary field elements that are included in the
    /// transaction but don't affect account state. They can be used for
    /// off-chain indexing and logging.
    pub events: Events<Fp>,

    /// Actions (sequenced events) emitted by this account update.
    ///
    /// Unlike events, actions are accumulated in the account's action state
    /// and can be processed by subsequent zkApp transactions.
    pub actions: Actions<Fp>,

    /// Arbitrary field element for zkApp-specific data.
    ///
    /// This field is included in the account update hash and can be used
    /// to pass data to the zkApp's verification logic.
    pub call_data: Fp,

    /// Preconditions that must be satisfied for this update to succeed.
    ///
    /// Includes network state preconditions (blockchain length, slot, etc.)
    /// and account state preconditions (balance, nonce, app state, etc.).
    pub preconditions: Preconditions<Pk, Fp>,

    /// Whether to use the full transaction commitment for signing.
    ///
    /// When `true`, signatures cover all account updates in the command.
    /// When `false`, signatures only cover this update and its descendants.
    pub use_full_commitment: bool,

    /// Whether to implicitly pay the account creation fee.
    ///
    /// When `true` and this update creates a new account, the creation fee
    /// is automatically deducted from the balance change.
    pub implicit_account_creation_fee: bool,

    /// Token permission inheritance configuration.
    ///
    /// Controls whether this update can use custom tokens and under what
    /// conditions.
    pub may_use_token: MayUseToken,

    /// The kind of authorization used for this update.
    ///
    /// Must be consistent with the `authorization` field in the parent
    /// `AccountUpdate` structure.
    pub authorization_kind: AuthorizationKind<Fp>,
}

/// Updates to apply to an account's state.
///
/// Each field uses [`SetOrKeep`] to either set a new value or keep the
/// existing value unchanged. This allows partial updates where only specific
/// fields are modified.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (Update)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update<Pk, Fp> {
    /// Updates to the 8 app state field elements.
    ///
    /// zkApp accounts have 8 field elements of arbitrary state that can be
    /// used by the zkApp's smart contract logic.
    pub app_state: [SetOrKeep<Fp>; 8],

    /// Update to the account's delegate.
    ///
    /// The delegate receives staking rewards on behalf of this account.
    /// Only applicable for accounts using the default MINA token.
    pub delegate: SetOrKeep<Pk>,

    /// Update to the account's verification key.
    ///
    /// The verification key is used to verify zero-knowledge proofs for
    /// account updates authorized by proof.
    pub verification_key: SetOrKeep<VerificationKeyHash<Fp>>,

    /// Update to the account's permissions.
    ///
    /// Permissions control which operations require which types of
    /// authorization (proof, signature, or impossible).
    pub permissions: SetOrKeep<Permissions>,

    /// Update to the zkApp URI.
    ///
    /// A URI pointing to off-chain resources related to this zkApp,
    /// such as documentation or a web interface.
    pub zkapp_uri: SetOrKeep<ZkAppUri>,

    /// Update to the token symbol.
    ///
    /// A human-readable symbol for custom tokens (e.g., "USDC", "WETH").
    pub token_symbol: SetOrKeep<TokenSymbol>,

    /// Update to the account's timing (vesting schedule).
    ///
    /// Timing controls when tokens become available for spending,
    /// used for vesting schedules and time-locked tokens.
    pub timing: SetOrKeep<Timing>,

    /// Update to the voting-for field.
    ///
    /// Indicates which proposal or election this account is voting for
    /// in on-chain governance.
    pub voting_for: SetOrKeep<VotingFor<Fp>>,
}

/// A value that can be set to a new value or kept unchanged.
///
/// Used throughout account updates to allow partial modifications where
/// only specific fields are changed.
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_basic.ml` (SetOrKeep)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetOrKeep<T> {
    /// Set the field to a new value.
    Set(T),

    /// Keep the existing value unchanged.
    Keep,
}

impl<T> SetOrKeep<T> {
    /// Returns `true` if this is `Keep`.
    pub fn is_keep(&self) -> bool {
        matches!(self, Self::Keep)
    }

    /// Returns `true` if this is `Set`.
    pub fn is_set(&self) -> bool {
        !self.is_keep()
    }
}

/// Preconditions that must be satisfied for an account update to succeed.
///
/// Preconditions enable conditional transaction execution, where updates
/// only apply if the blockchain and account state match expected values.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (Preconditions)
#[derive(Debug, Clone, PartialEq)]
pub struct Preconditions<Pk, Fp> {
    /// Network state preconditions (blockchain length, slot, etc.).
    pub network: NetworkPreconditions<Fp>,

    /// Account state preconditions (balance, nonce, app state, etc.).
    pub account: AccountPreconditions<Pk, Fp>,

    /// Slot range during which this update is valid.
    ///
    /// The update will fail if applied outside this slot range.
    pub valid_while: Numeric<Slot>,
}

/// Network state preconditions.
///
/// These preconditions check the global blockchain state at the time
/// the transaction is applied.
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_precondition.ml` (Protocol_state)
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkPreconditions<Fp> {
    /// Expected hash of the snarked ledger.
    pub snarked_ledger_hash: OrIgnore<Fp>,

    /// Expected blockchain length (block height).
    pub blockchain_length: Numeric<Length>,

    /// Expected minimum window density.
    pub min_window_density: Numeric<Length>,

    /// Expected total currency in circulation.
    pub total_currency: Numeric<Amount>,

    /// Expected global slot since genesis.
    pub global_slot_since_genesis: Numeric<Slot>,

    /// Preconditions on the current staking epoch.
    pub staking_epoch_data: EpochData<Fp>,

    /// Preconditions on the next staking epoch.
    pub next_epoch_data: EpochData<Fp>,
}

/// Epoch data preconditions.
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_precondition.ml` (EpochData)
#[derive(Debug, Clone, PartialEq)]
pub struct EpochData<Fp> {
    /// Expected epoch ledger hash.
    pub ledger: EpochLedger<Fp>,

    /// Expected epoch seed.
    pub seed: OrIgnore<Fp>,

    /// Expected start checkpoint.
    pub start_checkpoint: OrIgnore<Fp>,

    /// Expected lock checkpoint.
    pub lock_checkpoint: OrIgnore<Fp>,

    /// Expected epoch length.
    pub epoch_length: Numeric<Length>,
}

/// Epoch ledger preconditions.
#[derive(Debug, Clone, PartialEq)]
pub struct EpochLedger<Fp> {
    /// Expected ledger hash.
    pub hash: OrIgnore<Fp>,

    /// Expected total currency in the epoch ledger.
    pub total_currency: Numeric<Amount>,
}

/// Account state preconditions.
///
/// These preconditions check the state of the target account at the time
/// the update is applied.
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_precondition.ml` (Account)
#[derive(Debug, Clone, PartialEq)]
pub struct AccountPreconditions<Pk, Fp> {
    /// Expected account balance range.
    pub balance: Numeric<Balance>,

    /// Expected account nonce range.
    pub nonce: Numeric<Nonce>,

    /// Expected receipt chain hash.
    pub receipt_chain_hash: OrIgnore<Fp>,

    /// Expected delegate public key.
    pub delegate: OrIgnore<Pk>,

    /// Expected app state values (8 field elements).
    pub state: [OrIgnore<Fp>; 8],

    /// Expected action state (for sequenced events).
    pub action_state: OrIgnore<Fp>,

    /// Expected proved state flag.
    pub proved_state: OrIgnore<bool>,

    /// Expected "is new account" flag.
    pub is_new: OrIgnore<bool>,
}

/// A precondition that either checks a value or ignores it.
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_basic.ml` (OrIgnore)
#[derive(Debug, Clone, PartialEq)]
pub enum OrIgnore<T> {
    /// Check that the value matches.
    Check(T),

    /// Ignore this precondition (always passes).
    Ignore,
}

/// A numeric precondition specifying a valid range.
///
/// The precondition passes if the actual value falls within
/// `[lower, upper]` (inclusive).
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_precondition.ml` (Numeric)
pub type Numeric<T> = OrIgnore<ClosedInterval<T>>;

/// A closed interval `[lower, upper]`.
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_precondition.ml` (ClosedInterval)
#[derive(Debug, Clone, PartialEq)]
pub struct ClosedInterval<T> {
    /// The lower bound (inclusive).
    pub lower: T,

    /// The upper bound (inclusive).
    pub upper: T,
}

/// The kind of authorization used for an account update.
///
/// This must match the actual authorization provided in the `AccountUpdate`.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (AuthorizationKind)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationKind<Fp> {
    /// No authorization provided.
    ///
    /// Only valid for operations that don't require authorization
    /// based on the account's permissions.
    NoneGiven,

    /// Authorized by a cryptographic signature.
    Signature,

    /// Authorized by a zero-knowledge proof.
    ///
    /// The field element is the hash of the verification key that
    /// will verify the proof.
    Proof(Fp),
}

/// Token permission configuration for account updates.
///
/// Controls whether and how an account update can interact with custom tokens.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (MayUseToken)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MayUseToken {
    /// Cannot use custom tokens.
    No,

    /// Can inherit token permissions from parent updates.
    ParentsOwnToken,

    /// Can use tokens from any parent in the update tree.
    InheritFromParent,
}

/// Events emitted by an account update.
///
/// Events are arbitrary data included in the transaction for off-chain
/// indexing. They don't affect on-chain state.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (Events)
#[derive(Debug, Clone, PartialEq)]
pub struct Events<Fp>(pub Vec<Event<Fp>>);

/// A single event, consisting of field elements.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (Event)
#[derive(Debug, Clone, PartialEq)]
pub struct Event<Fp>(pub Vec<Fp>);

/// Actions (sequenced events) emitted by an account update.
///
/// Unlike events, actions are accumulated in the account's action state
/// and can be processed by subsequent transactions.
///
/// # References
///
/// OCaml: `src/lib/mina_base/zkapp_account.ml` (Actions)
#[derive(Debug, Clone, PartialEq)]
pub struct Actions<Fp>(pub Vec<Event<Fp>>);

/// Timing information for token vesting schedules.
///
/// Controls when tokens become available for spending through a
/// cliff and vesting schedule.
///
/// # References
///
/// OCaml: `src/lib/mina_base/account_update.ml` (Timing)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timing {
    /// The initial minimum balance that must be maintained.
    pub initial_minimum_balance: Balance,

    /// The slot at which the cliff occurs.
    ///
    /// Before this slot, `initial_minimum_balance` tokens are locked.
    pub cliff_time: Slot,

    /// The amount released at the cliff.
    pub cliff_amount: Amount,

    /// The number of slots between vesting releases.
    pub vesting_period: SlotSpan,

    /// The amount released at each vesting period.
    pub vesting_increment: Amount,
}

// ============================================================================
// Primitive types
// ============================================================================

/// Transaction memo field (34 bytes).
///
/// The memo has a specific format:
/// - Byte 0: Tag byte (0x01 for text, 0x02 for binary)
/// - Byte 1: Length of the payload (0-32)
/// - Bytes 2-33: Payload (user data, padded with zeros)
///
/// # References
///
/// OCaml: `src/lib/mina_base/signed_command_memo.ml`
#[derive(Clone, PartialEq, Eq)]
pub struct Memo(pub [u8; 34]);

impl core::fmt::Debug for Memo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Display the memo as a string if it's valid UTF-8 text
        if self.0[0] == 0x01 {
            let len = self.0[1] as usize;
            if len <= 32 {
                if let Ok(s) = core::str::from_utf8(&self.0[2..2 + len]) {
                    return write!(f, "Memo({:?})", s);
                }
            }
        }
        write!(f, "Memo({:?})", &self.0[..])
    }
}

/// A cryptographic signature (r, s components as field elements).
///
/// Signatures in Mina use the Schnorr signature scheme over the Pallas curve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// The r component of the signature.
    pub rx: [u8; 32],

    /// The s component of the signature.
    pub s: [u8; 32],
}

/// Transaction fee in nanomina (1 MINA = 10^9 nanomina).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fee(pub u64);

/// Token amount in the smallest unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(pub u64);

/// Account balance in the smallest unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Balance(pub u64);

/// Account nonce (transaction sequence number).
///
/// Incremented with each transaction from the account to prevent replay attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nonce(pub u32);

/// Global slot number since genesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot(pub u32);

/// A span of slots (duration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotSpan(pub u32);

/// Blockchain length (block height).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Length(pub u32);

/// A signed value with magnitude and sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signed<T> {
    /// The absolute value.
    pub magnitude: T,

    /// The sign (positive or negative).
    pub sgn: Sgn,
}

/// Sign of a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sgn {
    /// Positive value.
    Pos,

    /// Negative value.
    Neg,
}

/// Token identifier.
///
/// The default token ID represents MINA. Custom tokens have unique IDs
/// derived from the token owner's account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenId<Fp>(pub Fp);

/// Hash of a verification key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationKeyHash<Fp>(pub Fp);

/// zkApp URI for off-chain resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkAppUri(pub Vec<u8>);

/// Human-readable token symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenSymbol(pub Vec<u8>);

/// Voting-for field (governance).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VotingFor<Fp>(pub Fp);

/// Account permissions controlling operation authorization.
///
/// Each permission specifies what authorization is required for a
/// particular operation.
///
/// # References
///
/// OCaml: `src/lib/mina_base/permissions.ml`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permissions {
    /// Permission to edit the account state.
    pub edit_state: AuthRequired,

    /// Permission to send tokens.
    pub send: AuthRequired,

    /// Permission to receive tokens.
    pub receive: AuthRequired,

    /// Permission to access the account.
    pub access: AuthRequired,

    /// Permission to set the delegate.
    pub set_delegate: AuthRequired,

    /// Permission to set permissions.
    pub set_permissions: AuthRequired,

    /// Permission to set the verification key.
    pub set_verification_key: SetVerificationKeyPerm,

    /// Permission to set the zkApp URI.
    pub set_zkapp_uri: AuthRequired,

    /// Permission to edit action state.
    pub edit_action_state: AuthRequired,

    /// Permission to set the token symbol.
    pub set_token_symbol: AuthRequired,

    /// Permission to increment the nonce.
    pub increment_nonce: AuthRequired,

    /// Permission to set voting-for.
    pub set_voting_for: AuthRequired,

    /// Permission to set timing.
    pub set_timing: AuthRequired,
}

/// Permission for setting the verification key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetVerificationKeyPerm {
    /// The authorization required.
    pub auth: AuthRequired,

    /// The transaction version this permission applies to.
    pub txn_version: u32,
}

/// Authorization requirement for an operation.
///
/// # References
///
/// OCaml: `src/lib/mina_base/permissions.ml` (Auth_required)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthRequired {
    /// No authorization required.
    None,

    /// Either signature or proof is acceptable.
    Either,

    /// Only a zero-knowledge proof is acceptable.
    Proof,

    /// Only a signature is acceptable.
    Signature,

    /// Operation is not permitted (impossible).
    Impossible,
}

/// Stack hash for call forest commitment.
pub type StackHash = [u8; 32];

/// Account update digest for commitment.
pub type AccountUpdateDigest = [u8; 32];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_or_keep() {
        let set: SetOrKeep<u32> = SetOrKeep::Set(42);
        let keep: SetOrKeep<u32> = SetOrKeep::Keep;

        assert!(set.is_set());
        assert!(!set.is_keep());
        assert!(keep.is_keep());
        assert!(!keep.is_set());
    }

    #[test]
    fn test_memo_debug() {
        // Text memo with "hello"
        let mut memo_bytes = [0u8; 34];
        memo_bytes[0] = 0x01; // text tag
        memo_bytes[1] = 5; // length
        memo_bytes[2..7].copy_from_slice(b"hello");

        let memo = Memo(memo_bytes);
        let debug_str = alloc::format!("{:?}", memo);
        assert!(debug_str.contains("hello"));
    }
}
