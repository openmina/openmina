SELECT
    b.state_hash AS "state_hash!"
FROM blocks b
ORDER BY b.id DESC
LIMIT 1;