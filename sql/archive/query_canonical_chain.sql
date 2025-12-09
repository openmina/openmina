WITH RECURSIVE chain AS (
    (SELECT * FROM blocks WHERE state_hash = $1)
        UNION ALL
        SELECT b.* FROM blocks b
        INNER JOIN chain
        ON b.id = chain.parent_id AND chain.id <> chain.parent_id
    )

    SELECT 
        c.id AS "id!", 
        c.state_hash AS "state_hash!", 
        c.height AS "height!", 
        c.timestamp AS "timestamp!", 
        c.chain_status AS "chain_status!: ChainStatus",
        pk_creator.value AS "creator_key",
        pk_winner.value AS "winner_key",
        c.global_slot_since_genesis AS "global_slot_since_genesis!",
        c.global_slot_since_hard_fork AS "global_slot_since_hard_fork!",
        c.parent_id
    FROM 
        chain c
    JOIN 
        public_keys pk_creator ON c.creator_id = pk_creator.id
    JOIN 
        public_keys pk_winner ON c.block_winner_id = pk_winner.id
    WHERE 
        c.global_slot_since_hard_fork BETWEEN $2 AND $3