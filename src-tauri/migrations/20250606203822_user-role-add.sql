create table users_new
(
    id            INTEGER
        primary key autoincrement,
    username      TEXT                               not null
        unique,
    role          TEXT                               not null,
    password_hash TEXT                               not null,
    created_at    DATETIME default CURRENT_TIMESTAMP not null,
    updated_at    DATETIME default CURRENT_TIMESTAMP not null
);

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;
