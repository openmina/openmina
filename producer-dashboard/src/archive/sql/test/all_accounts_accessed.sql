SELECT 
    aa.ledger_index,
    aa.block_id,
    aa.account_identifier_id,
    aa.token_symbol_id,
    aa.balance,
    aa.nonce,
    aa.receipt_chain_hash,
    aa.delegate_id,
    aa.voting_for_id,
    aa.timing_id,
    aa.permissions_id,
    aa.zkapp_id
FROM accounts_accessed aa;
