SELECT
    id,
    command_type AS "command_type: UserCommandType",
    fee_payer_id,
    source_id,
    receiver_id,
    nonce,
    amount,
    fee,
    valid_until,
    memo,
    hash
FROM user_commands;
