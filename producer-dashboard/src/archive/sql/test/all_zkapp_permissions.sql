SELECT
    id,
    edit_state AS "edit_state: ZkappAuthRequiredType",
    send AS "send: ZkappAuthRequiredType",
    receive AS "receive: ZkappAuthRequiredType",
    access AS "access: ZkappAuthRequiredType",
    set_delegate AS "set_delegate: ZkappAuthRequiredType",
    set_permissions AS "set_permissions: ZkappAuthRequiredType",
    set_verification_key_auth AS "set_verification_key_auth: ZkappAuthRequiredType",
    set_verification_key_txn_version, 
    set_zkapp_uri AS "set_zkapp_uri: ZkappAuthRequiredType",
    edit_action_state AS "edit_action_state: ZkappAuthRequiredType",
    set_token_symbol AS "set_token_symbol: ZkappAuthRequiredType",
    increment_nonce AS "increment_nonce: ZkappAuthRequiredType",
    set_voting_for AS "set_voting_for: ZkappAuthRequiredType",
    set_timing AS "set_timing: ZkappAuthRequiredType"
FROM zkapp_permissions;
