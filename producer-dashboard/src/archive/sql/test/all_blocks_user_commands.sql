SELECT 
    bc.block_id,
    bc.user_command_id,
    bc.sequence_no,
    bc.status AS "status: TransactionStatus",
    bc.failure_reason
FROM blocks_user_commands bc;
