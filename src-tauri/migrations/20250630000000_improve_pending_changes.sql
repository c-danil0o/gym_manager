-- Migration to improve pending_changes table for better sync performance
--
DROP TRIGGER IF EXISTS users_sync_insert;
DROP TRIGGER IF EXISTS users_sync_update;
DROP TRIGGER IF EXISTS users_sync_delete;
DROP TRIGGER IF EXISTS entry_logs_sync_insert;
DROP TRIGGER IF EXISTS entry_logs_sync_update;
DROP TRIGGER IF EXISTS entry_logs_sync_delete;

-- Add status field to track processing state
ALTER TABLE pending_changes ADD COLUMN status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'failed'));

-- Create optimized indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_pending_changes_status_created_at ON pending_changes (status, created_at);
CREATE INDEX IF NOT EXISTS idx_pending_changes_table_status ON pending_changes (table_name, status);
CREATE INDEX IF NOT EXISTS idx_pending_changes_retry_count ON pending_changes (retry_count);

-- Remove the old individual indexes that are now redundant
DROP INDEX IF EXISTS idx_pending_changes_created_at;
DROP INDEX IF EXISTS idx_pending_changes_table_operation;
