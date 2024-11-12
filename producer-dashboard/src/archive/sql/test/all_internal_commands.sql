SELECT
    id,
    command_type AS "command_type: InternalCommandType",
    receiver_id,
    fee,
    hash
FROM internal_commands;
