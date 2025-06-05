CREATE TABLE members_temp
(
    id            INTEGER
        PRIMARY KEY AUTOINCREMENT,
    card_id       TEXT
        UNIQUE,
    short_card_id TEXT
        UNIQUE,
    first_name    TEXT                               NOT NULL,
    last_name     TEXT                               NOT NULL,
    email         TEXT,
    phone         TEXT,
    date_of_birth DATE,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at    DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    is_deleted    BOOLEAN  DEFAULT FALSE             NOT NULL
);

INSERT INTO members_temp (
    id, card_id, short_card_id, first_name, last_name,
    email, phone, date_of_birth, created_at, updated_at, is_deleted
)
SELECT
    id, card_id, short_card_id, first_name, last_name,
    email, phone, date_of_birth, created_at, updated_at, is_deleted
FROM members;

DROP TABLE members;

ALTER TABLE members_temp RENAME TO members;

CREATE INDEX idx_member_card_id
    ON members (card_id);

CREATE INDEX idx_member_updated_at
    ON members (updated_at);

CREATE INDEX idx_short_member_card_id
    ON members (short_card_id);
