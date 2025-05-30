-- Add migration script here
ALTER TABLE entry_logs
    ADD card_id text;
