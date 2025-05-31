-- Add migration script here
-- Create new table with updated CHECK constraint including the new status types
CREATE TABLE entry_logs_new (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    member_id     INTEGER                            -- Removed NOT NULL constraint
        REFERENCES members(id) ON DELETE CASCADE,
    membership_id INTEGER
        REFERENCES memberships(id) ON DELETE SET NULL,
    card_id       TEXT                               NOT NULL,
    entry_time    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    status        TEXT                               NOT NULL,
    notes         TEXT,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    -- Updated CHECK constraint with all status values including new ones
    CHECK (status IN (
        'allowed',
        'denied_member_not_found',
        'denied_no_membership',
        'denied_no_visits_left',
        'denied_membership_expired',
        'denied_membership_not_active_yet',
        'denied_membership_inactive',
        'denied_membership_suspended',
        'denied_membership_invalid_status',
        'denied_already_checked_in',
        'denied_after_hours',
        'error_updating_membership',
        'error'
    ))
);

-- Copy existing data from old table to new table
INSERT INTO entry_logs_new (
    id, member_id, membership_id, card_id, entry_time,
    status, notes, created_at
)
SELECT
    id, member_id, membership_id, card_id, entry_time,
    status, notes, created_at
FROM entry_logs;

-- Drop the old table
DROP TABLE entry_logs;

-- Rename the new table
ALTER TABLE entry_logs_new RENAME TO entry_logs;

-- Recreate all indexes
CREATE INDEX idx_entry_log_entry_time ON entry_logs (entry_time);
CREATE INDEX idx_entry_log_member_id ON entry_logs (member_id);
CREATE INDEX idx_entry_log_status ON entry_logs (status);
CREATE INDEX idx_entry_log_card_id ON entry_logs (card_id);
CREATE INDEX idx_entry_log_membership_id ON entry_logs (membership_id);
CREATE INDEX idx_entry_log_search ON entry_logs (status, entry_time DESC);
