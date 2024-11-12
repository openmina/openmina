SELECT 
    bc.block_id,
    bc.internal_command_id,
    bc.sequence_no,
    bc.secondary_sequence_no,
    bc.status AS "status: TransactionStatus",
    bc.failure_reason
FROM blocks_internal_commands bc;
