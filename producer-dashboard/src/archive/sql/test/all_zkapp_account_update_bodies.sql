SELECT
    id,
    account_identifier_id,
    update_id,
    balance_change,
    increment_nonce,
    events_id,
    actions_id,
    call_data_id,
    call_depth,
    zkapp_network_precondition_id,
    zkapp_account_precondition_id,
    zkapp_valid_while_precondition_id,
    use_full_commitment,
    implicit_account_creation_fee,
    may_use_token AS "may_use_token: MayUseToken",
    authorization_kind AS "authorization_kind: AuthorizationKind",
    verification_key_hash_id
FROM zkapp_account_update_body;
