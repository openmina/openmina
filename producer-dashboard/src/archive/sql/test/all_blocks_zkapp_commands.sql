SELECT
    bc.block_id,
    bc.zkapp_command_id,
    bc.sequence_no,
    bc.status AS "status: TransactionStatus",
    bc.failure_reasons_ids
FROM blocks_zkapp_commands bc;
