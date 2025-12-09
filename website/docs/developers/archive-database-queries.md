---
title: Archive Database Queries
description: SQL queries and database analysis for Mina Rust archive nodes
sidebar_position: 7
---

# Archive Database Queries

This guide provides comprehensive SQL queries and analysis techniques for
querying Mina Rust archive node databases. Archive nodes store complete
blockchain history in a structured PostgreSQL database with over 45 tables.

## Prerequisites

- Running archive node with PostgreSQL database
- Database credentials (default: user `postgres`, database `archive`)
- PostgreSQL client tools installed

## Database Connection

### Direct Connection

```bash
# Connect using psql (requires PostgreSQL client installed)
psql -h localhost -p 5432 -U postgres -d archive

# Or connect from within the Docker environment
# Note: postgres-mina-rust is the container name from the archive node setup
# See: https://o1-labs.github.io/mina-rust/node-operators/archive-node
docker exec -it postgres-mina-rust psql -U postgres -d archive
```

## Database Schema

### Schema Exploration

```sql
-- List all tables in the archive database
\dt

-- Get detailed information about table columns
\d table_name

-- Example: Get structure of the blocks table
\d blocks

-- Show all tables with their sizes
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(tablename::text)) as size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(tablename::text) DESC;
```

### Key Tables

The archive database contains these primary tables:

- **`blocks`** - Block headers and metadata
- **`user_commands`** - Payment and delegation transactions
- **`internal_commands`** - Fee transfers, coinbase rewards
- **`zkapp_commands`** - zkApp transactions and proof data
- **`accounts_accessed`** - Account state changes
- **`accounts_created`** - New account creations
- **`blocks_user_commands`** - Links blocks to user commands
- **`blocks_internal_commands`** - Links blocks to internal commands
- **`public_keys`** - Public key identifiers
- **`zkapp_*`** - Various zkApp-related tables for smart contract data

### Schema Files

The complete database schema definitions can be found in the repository:

- **Schema file**:
  [`sql/archive/archive_schema.sql`](https://github.com/o1-labs/mina-rust/blob/develop/sql/archive/archive_schema.sql) -
  Complete PostgreSQL database schema for the archive node
- **Query examples**:
  [`sql/archive/`](https://github.com/o1-labs/mina-rust/tree/develop/sql/archive) -
  Pre-built queries for common operations such as querying blocks in slot
  ranges, canonical chain, latest blocks, and producer-specific blocks

## Common SQL Queries

### Block Information

```sql
-- Get recent blocks
SELECT
    b.height,
    b.state_hash,
    b.parent_hash,
    pk.value as creator,
    b.timestamp
FROM blocks b
JOIN public_keys pk ON b.creator_id = pk.id
ORDER BY b.height DESC
LIMIT 10;

-- Get block statistics by creator
SELECT
    pk.value as creator,
    COUNT(*) as blocks_produced,
    MIN(b.height) as first_block,
    MAX(b.height) as latest_block
FROM blocks b
JOIN public_keys pk ON b.creator_id = pk.id
GROUP BY pk.value
ORDER BY blocks_produced DESC;
```

### Transaction Analysis

```sql
-- Get recent payments
SELECT
    b.height,
    b.timestamp,
    pk_source.value as source,
    pk_receiver.value as receiver,
    uc.amount,
    uc.fee
FROM user_commands uc
JOIN blocks_user_commands buc ON uc.id = buc.user_command_id
JOIN blocks b ON buc.block_id = b.id
JOIN public_keys pk_source ON uc.source_id = pk_source.id
JOIN public_keys pk_receiver ON uc.receiver_id = pk_receiver.id
WHERE uc.command_type = 'payment'
ORDER BY b.height DESC
LIMIT 20;

-- Transaction volume analysis
SELECT
    DATE(to_timestamp(b.timestamp::bigint / 1000)) as date,
    COUNT(uc.id) as tx_count,
    SUM(uc.amount::bigint) as total_volume,
    AVG(uc.fee::bigint) as avg_fee
FROM user_commands uc
JOIN blocks_user_commands buc ON uc.id = buc.user_command_id
JOIN blocks b ON buc.block_id = b.id
WHERE uc.command_type = 'payment'
GROUP BY DATE(to_timestamp(b.timestamp::bigint / 1000))
ORDER BY date DESC;
```

### Account Analysis

```sql
-- Most active accounts
SELECT
    pk.value as public_key,
    COUNT(*) as transaction_count
FROM user_commands uc
JOIN public_keys pk ON uc.source_id = pk.id
GROUP BY pk.value
ORDER BY transaction_count DESC
LIMIT 10;

-- Account balance history (requires account state tracking)
SELECT
    aa.account_identifier_id,
    pk.value as public_key,
    aa.balance,
    b.height,
    b.timestamp
FROM accounts_accessed aa
JOIN public_keys pk ON aa.account_identifier_id = pk.id
JOIN blocks b ON aa.block_id = b.id
WHERE pk.value = 'YOUR_PUBLIC_KEY_HERE'
ORDER BY b.height DESC;
```

## Analytics and Reporting

### Python Analysis Script

```python
import psycopg2
import pandas as pd
import matplotlib.pyplot as plt

# Connect to archive database
conn = psycopg2.connect(
    host="localhost",
    port=5432,
    database="archive",
    user="postgres",
    password="mina"
)

# Analyze block production over time
query = """
SELECT
    DATE(to_timestamp(timestamp::bigint / 1000)) as date,
    COUNT(*) as blocks_per_day,
    COUNT(DISTINCT creator_id) as unique_producers
FROM blocks
WHERE timestamp > extract(epoch from now() - interval '30 days') * 1000
GROUP BY date
ORDER BY date;
"""

df = pd.read_sql_query(query, conn)
print(df)

# Plot block production
df.plot(x='date', y='blocks_per_day', kind='line')
plt.title('Daily Block Production')
plt.show()
```

### Export Data for Analysis

```bash
# Export recent transactions to CSV
docker exec postgres-mina-rust psql -U postgres -d archive -c "\COPY (
  SELECT
    b.height,
    b.timestamp,
    pk_source.value as source,
    pk_receiver.value as receiver,
    uc.amount,
    uc.fee,
    uc.memo
  FROM user_commands uc
  JOIN blocks_user_commands buc ON uc.id = buc.user_command_id
  JOIN blocks b ON buc.block_id = b.id
  JOIN public_keys pk_source ON uc.source_id = pk_source.id
  JOIN public_keys pk_receiver ON uc.receiver_id = pk_receiver.id
  WHERE uc.command_type = 'payment'
  ORDER BY b.height DESC
  LIMIT 1000
) TO STDOUT WITH CSV HEADER" > transactions.csv
```

## Database Maintenance

### Backup and Restore

```bash
# Create database dump
docker exec postgres-mina-rust pg_dump -U postgres archive > archive_backup.sql

# Restore from backup
docker exec -i postgres-mina-rust psql -U postgres archive < archive_backup.sql
```

### Performance Optimization

```sql
-- Create indexes for common queries
CREATE INDEX CONCURRENTLY idx_blocks_height ON blocks(height);
CREATE INDEX CONCURRENTLY idx_blocks_timestamp ON blocks(timestamp);
CREATE INDEX CONCURRENTLY idx_user_commands_source ON user_commands(source_id);
CREATE INDEX CONCURRENTLY idx_user_commands_receiver ON user_commands(receiver_id);

-- Analyze table statistics
ANALYZE blocks;
ANALYZE user_commands;
ANALYZE accounts_accessed;
```

### Storage Management

```bash
# Check database size
docker exec postgres-mina-rust psql -U postgres -d archive -c "
  SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(tablename::text)) as size
  FROM pg_tables
  WHERE schemaname = 'public'
  ORDER BY pg_total_relation_size(tablename::text) DESC;
"

# Vacuum and analyze for performance
docker exec postgres-mina-rust psql -U postgres -d archive -c "VACUUM ANALYZE;"
```

## Advanced Queries

### Network Statistics

```sql
-- Block production distribution
SELECT
    pk.value as producer,
    COUNT(*) as blocks_produced,
    ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER(), 2) as percentage
FROM blocks b
JOIN public_keys pk ON b.creator_id = pk.id
GROUP BY pk.value
ORDER BY blocks_produced DESC;

-- Transaction fee analysis
SELECT
    percentile_cont(0.5) WITHIN GROUP (ORDER BY uc.fee::bigint) as median_fee,
    percentile_cont(0.75) WITHIN GROUP (ORDER BY uc.fee::bigint) as p75_fee,
    percentile_cont(0.95) WITHIN GROUP (ORDER BY uc.fee::bigint) as p95_fee,
    AVG(uc.fee::bigint) as avg_fee,
    MIN(uc.fee::bigint) as min_fee,
    MAX(uc.fee::bigint) as max_fee
FROM user_commands uc
WHERE uc.command_type = 'payment';
```

### zkApp Analytics

```sql
-- zkApp transaction volume
SELECT
    DATE(to_timestamp(b.timestamp::bigint / 1000)) as date,
    COUNT(zc.id) as zkapp_count
FROM zkapp_commands zc
JOIN blocks_zkapp_commands bzc ON zc.id = bzc.zkapp_command_id
JOIN blocks b ON bzc.block_id = b.id
GROUP BY DATE(to_timestamp(b.timestamp::bigint / 1000))
ORDER BY date DESC;

-- Most active zkApp accounts
SELECT
    pk.value as public_key,
    COUNT(*) as zkapp_transactions
FROM zkapp_commands zc
JOIN zkapp_fee_payer_body zfpb ON zc.zkapp_fee_payer_body_id = zfpb.id
JOIN public_keys pk ON zfpb.public_key_id = pk.id
GROUP BY pk.value
ORDER BY zkapp_transactions DESC
LIMIT 10;
```

## Use Cases

1. **Compliance and Auditing**: Track all transactions for regulatory compliance
2. **Analytics Dashboards**: Build real-time blockchain analytics
3. **Research**: Analyze network behavior, transaction patterns, and economics
4. **Block Explorers**: Power blockchain explorer websites
5. **Tax Reporting**: Generate transaction history for tax purposes
6. **Network Monitoring**: Track network health and validator performance

## Performance Considerations

- **Complex queries**: Use appropriate indexes and LIMIT clauses
- **Large result sets**: Consider pagination for web applications
- **Historical data**: Older data may be slower to query
- **Concurrent access**: Archive database can handle multiple read connections
- **Memory usage**: Large analytical queries may require sufficient RAM

## Troubleshooting

### Common Issues

**Connection refused**:

```bash
# Check if PostgreSQL container is running
docker ps | grep postgres

# Check container logs
docker logs postgres-mina-rust
```

**Permission denied**:

```bash
# Ensure proper database credentials
docker exec postgres-mina-rust psql -U postgres -l
```

**Query timeout**:

```sql
-- Add appropriate indexes for slow queries
-- Use EXPLAIN ANALYZE to understand query performance
EXPLAIN ANALYZE SELECT ...;
```

## Next Steps

- [GraphQL API Reference](./graphql-api) - Query blockchain data via GraphQL
- [Node Architecture](./architecture) - Understanding the archive system
- [Running Archive Nodes](../node-operators/archive-node) - Setting up archive
  infrastructure
