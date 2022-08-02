use mina_signer::CompressedPubKey;

use super::common::*;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/account.ml#L93
pub type TokenSymbol = String;

// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/permissions.mli#L49
#[derive(Clone, Debug)]
pub struct Permissions<Controller> {
    edit_state: Controller,
    send: Controller,
    receive: Controller,
    set_delegate: Controller,
    set_permissions: Controller,
    set_verification_key: Controller,
    set_zkapp_uri: Controller,
    edit_sequence_state: Controller,
    set_token_symbol: Controller,
    increment_nonce: Controller,
    set_voting_for: Controller,
}

impl Default for Permissions<AuthRequired> {
    fn default() -> Self {
        Self::user_default()
    }
}

impl Permissions<AuthRequired> {
    fn user_default() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: Signature,
            send: Signature,
            receive: None,
            set_delegate: Signature,
            set_permissions: Signature,
            set_verification_key: Signature,
            set_zkapp_uri: Signature,
            edit_sequence_state: Signature,
            set_token_symbol: Signature,
            increment_nonce: Signature,
            set_voting_for: Signature,
        }
    }

    fn empty() -> Self {
        use AuthRequired::*;
        Self {
            edit_state: None,
            send: None,
            receive: None,
            set_delegate: None,
            set_permissions: None,
            set_verification_key: None,
            set_zkapp_uri: None,
            edit_sequence_state: None,
            set_token_symbol: None,
            increment_nonce: None,
            set_voting_for: None,
        }
    }
}

// TODO: Fill this struct
// https://github.com/MinaProtocol/mina/blob/develop/src/lib/mina_base/zkapp_account.ml#L148-L170
#[derive(Clone, Debug)]
pub struct ZkAppAccount {
    app_state: Vec<u8>,
    verification: (),
    zkapp_version: u32,
    sequence_state: (),
    last_sequence_slot: Slot,
    proved_state: bool,
}

// https://github.com/MinaProtocol/mina/blob/1765ba6bdfd7c454e5ae836c49979fa076de1bea/src/lib/mina_base/account.ml#L368
#[derive(Clone, Debug)]
pub struct Account {
    pub public_key: CompressedPubKey,         // Public_key.Compressed.t
    pub token_id: TokenId,                    // Token_id.t
    pub token_permissions: TokenPermissions,  // Token_permissions.t
    pub balance: Balance,                     // Balance.t
    pub nonce: Nonce,                         // Nonce.t
    pub receipt_chain_hash: ReceiptChainHash, // Receipt.Chain_hash.t
    pub delegate: Option<CompressedPubKey>,   // Public_key.Compressed.t option
    pub voting_for: VotingFor,                // State_hash.t
    pub timing: Timing,                       // Timing.t
    pub permissions: Permissions<AuthRequired>, // Permissions.t
    // Below fields are for `develop` branch
    pub token_symbol: TokenSymbol,   // Token_symbol.t
    pub zkapp: Option<ZkAppAccount>, // Zkapp_account.t
    pub zkapp_uri: String,           // string
}
