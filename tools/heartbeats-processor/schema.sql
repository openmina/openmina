CREATE TABLE IF NOT EXISTS public_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_key TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS submitter_counts (
    public_key_id INTEGER PRIMARY KEY,
    count INTEGER NOT NULL,
    last_seen INTEGER NOT NULL,  -- Unix timestamp
    updated_at INTEGER DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id)
);

CREATE TABLE IF NOT EXISTS processing_state (
    id INTEGER PRIMARY KEY,
    last_processed_time INTEGER NOT NULL  -- Unix timestamp
);

INSERT OR IGNORE INTO processing_state (id, last_processed_time)
VALUES (1, 0);

CREATE TABLE IF NOT EXISTS time_windows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time INTEGER NOT NULL,  -- Unix timestamp
    end_time INTEGER NOT NULL,    -- Unix timestamp
    disabled BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(start_time, end_time)
);

CREATE TABLE IF NOT EXISTS heartbeat_presence (
    window_id INTEGER NOT NULL,
    public_key_id INTEGER NOT NULL,
    best_tip_hash TEXT NOT NULL,
    best_tip_height INTEGER NOT NULL,
    best_tip_global_slot INTEGER NOT NULL,
    heartbeat_time INTEGER NOT NULL,
    disabled BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (window_id, public_key_id),
    FOREIGN KEY (window_id) REFERENCES time_windows(id),
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id)
);

CREATE TABLE IF NOT EXISTS submitter_scores (
    public_key_id INTEGER PRIMARY KEY,
    score INTEGER NOT NULL DEFAULT 0,
    last_updated INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    blocks_produced INTEGER NOT NULL DEFAULT 0,
    last_heartbeat INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id)
);

CREATE TABLE IF NOT EXISTS produced_blocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    window_id INTEGER NOT NULL,
    public_key_id INTEGER NOT NULL,
    block_hash TEXT NOT NULL,
    block_height INTEGER NOT NULL,
    block_global_slot INTEGER NOT NULL,
    block_data_blob TEXT,  -- Raw block data in base64-encoded binprot format
    validated BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(public_key_id, block_hash),
    FOREIGN KEY (window_id, public_key_id) REFERENCES heartbeat_presence(window_id, public_key_id),
    FOREIGN KEY (window_id) REFERENCES time_windows(id),
    FOREIGN KEY (public_key_id) REFERENCES public_keys(id)
);

-- Index for time window queries
CREATE INDEX IF NOT EXISTS idx_time_windows_start_end 
ON time_windows(start_time, end_time);

-- Index for public key lookups
CREATE INDEX IF NOT EXISTS idx_public_keys_key 
ON public_keys(public_key);

-- Index for presence queries by window
CREATE INDEX IF NOT EXISTS idx_heartbeat_presence_window 
ON heartbeat_presence(window_id);

-- Index for presence queries by public key
CREATE INDEX IF NOT EXISTS idx_heartbeat_presence_pubkey 
ON heartbeat_presence(public_key_id);

-- Index for presence queries by global slot
CREATE INDEX IF NOT EXISTS idx_heartbeat_presence_global_slot
ON heartbeat_presence(best_tip_global_slot);

-- Index for presence queries by best tip height
CREATE INDEX IF NOT EXISTS idx_heartbeat_presence_height
ON heartbeat_presence(best_tip_height);

-- Combined index for height queries with disabled flag
CREATE INDEX IF NOT EXISTS idx_heartbeat_presence_window_disabled_height
ON heartbeat_presence(window_id, disabled, best_tip_height);

-- Combined index for disabled flag, window and global slot lookups
CREATE INDEX IF NOT EXISTS idx_heartbeat_presence_window_disabled_global_slot
ON heartbeat_presence(window_id, disabled, best_tip_global_slot);

-- Index for heartbeat time queries
CREATE INDEX IF NOT EXISTS idx_heartbeat_presence_time
ON heartbeat_presence(heartbeat_time);

-- Index for submitter counts lookup
CREATE INDEX IF NOT EXISTS idx_submitter_counts_last_seen 
ON submitter_counts(last_seen);

-- Index for submitter scores lookup
CREATE INDEX IF NOT EXISTS idx_submitter_scores_score 
ON submitter_scores(score DESC);

-- Index for produced blocks queries by window
CREATE INDEX IF NOT EXISTS idx_produced_blocks_window
ON produced_blocks(window_id);

-- Index for produced blocks queries by public key
CREATE INDEX IF NOT EXISTS idx_produced_blocks_pubkey
ON produced_blocks(public_key_id);

-- Index for produced blocks queries by block hash
CREATE INDEX IF NOT EXISTS idx_produced_blocks_hash
ON produced_blocks(block_hash);

-- Combined index for window and public key lookups
CREATE INDEX IF NOT EXISTS idx_produced_blocks_window_pubkey
ON produced_blocks(window_id, public_key_id);

-- Index for global slot queries
CREATE INDEX IF NOT EXISTS idx_produced_blocks_global_slot
ON produced_blocks(block_global_slot);
