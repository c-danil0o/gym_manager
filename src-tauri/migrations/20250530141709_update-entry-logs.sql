-- Add migration script here

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

    -- Updated CHECK constraint with all status values from the code
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
        'error_updating_membership'
    ))
);

DROP TABLE entry_logs;

ALTER TABLE entry_logs_new RENAME TO entry_logs;

CREATE INDEX idx_entry_log_entry_time ON entry_logs (entry_time);

CREATE INDEX idx_entry_log_member_id ON entry_logs (member_id);
CREATE INDEX idx_entry_log_status ON entry_logs (status);
CREATE INDEX idx_entry_log_card_id ON entry_logs (card_id);
CREATE INDEX idx_entry_log_membership_id ON entry_logs (membership_id);

CREATE INDEX idx_entry_log_search ON entry_logs (status, entry_time DESC);
