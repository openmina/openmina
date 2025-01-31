SELECT 
    b.id, 
    b.state_hash, 
    b.height, 
    b.timestamp, 
    b.chain_status AS "chain_status: ChainStatus",
    pk_creator.value AS "creator_key",
    pk_winner.value AS "winner_key",
    b.global_slot_since_genesis,
    b.global_slot_since_hard_fork,
    b.parent_id
FROM 
    blocks b
JOIN 
    public_keys pk_creator ON b.creator_id = pk_creator.id
JOIN 
    public_keys pk_winner ON b.block_winner_id = pk_winner.id
WHERE 
    b.global_slot_since_hard_fork BETWEEN $1 AND $2