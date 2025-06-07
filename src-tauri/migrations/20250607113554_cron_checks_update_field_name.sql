-- Add migration script here
drop table if exists cron_checks;
create table cron_checks
(
    id                          INTEGER
        primary key autoincrement,
    last_check_time             DATETIME                           not null,
    last_success_time           DATETIME,
    check_type                        TEXT                               not null,
    status                      TEXT                               not null,
    error_message               TEXT,
    created_at                  DATETIME default CURRENT_TIMESTAMP not null,
    updated_at                  DATETIME default CURRENT_TIMESTAMP not null,
    check (status IN (
                      'checked_no_changes',
                      'upload_success',
                      'upload_failed',
                      'membership_update_success',
                      'membership_update_failed'
        ))
    check (check_type IN (
                      'backup',
                      'membership'
        ))
);
