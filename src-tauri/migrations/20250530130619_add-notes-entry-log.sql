-- Add migration script here
ALTER TABLE entry_logs
    ADD notes text;
