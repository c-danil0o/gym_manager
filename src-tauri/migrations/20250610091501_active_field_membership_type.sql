-- Add migration script here
alter table membership_types
    add is_active BOOLEAN default TRUE not null;
