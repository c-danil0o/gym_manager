CREATE TABLE entry_logs_new (
    id            INTEGER
        PRIMARY KEY AUTOINCREMENT,
    member_id     INTEGER
        REFERENCES members
            ON DELETE CASCADE,
    membership_id INTEGER
        REFERENCES memberships
            ON DELETE SET NULL,
    card_id       TEXT,  -- Removed NOT NULL constraint
    member_name   TEXT,  -- New field added
    entry_time    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    status        TEXT                               NOT NULL,
    notes         TEXT,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CHECK (status IN (
        'allowed',
        'allowed_single',  -- New status added
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

DROP TABLE entry_logs;

ALTER TABLE entry_logs_new RENAME TO entry_logs;

CREATE INDEX idx_entry_log_card_id
    ON entry_logs (card_id);

CREATE INDEX idx_entry_log_entry_time
    ON entry_logs (entry_time);

CREATE INDEX idx_entry_log_search
    ON entry_logs (status ASC, entry_time DESC);

CREATE INDEX idx_entry_log_status
    ON entry_logs (status);
