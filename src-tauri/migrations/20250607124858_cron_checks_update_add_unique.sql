-- Add migration script here
drop table if exists cron_checks;
create table cron_checks
(
    id                          INTEGER
        primary key autoincrement,
    last_check_time             DATETIME                           not null,
    check_type                  TEXT UNIQUE                        not null,
    status                      TEXT                               not null,
    created_at                  DATETIME default CURRENT_TIMESTAMP not null,
    updated_at                  DATETIME default CURRENT_TIMESTAMP not null,
    check (status IN (
                      'success',
                      'fail',
                      'no_changes'
        ))
    check (check_type IN (
                      'backup',
                      'membership'
        ))
);
