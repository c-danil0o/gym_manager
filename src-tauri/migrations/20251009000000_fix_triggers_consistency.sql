-- Migration to fix triggers consistency with updated table schemas
-- This addresses issues where triggers don't include all current table fields

-- First, drop all existing triggers to recreate them with correct field mappings
DROP TRIGGER IF EXISTS members_sync_insert;
DROP TRIGGER IF EXISTS members_sync_update;
DROP TRIGGER IF EXISTS members_sync_delete;

-- Recreate members triggers with the missing auth_user_id field
CREATE TRIGGER members_sync_insert
AFTER INSERT ON members
FOR EACH ROW
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('members', NEW.id, 'INSERT', json_object(
    'id', NEW.id,
    'card_id', NEW.card_id,
    'short_card_id', NEW.short_card_id,
    'first_name', NEW.first_name,
    'last_name', NEW.last_name,
    'email', NEW.email,
    'phone', NEW.phone,
    'date_of_birth', NEW.date_of_birth,
    'auth_user_id', NEW.auth_user_id,
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at,
    'is_deleted', NEW.is_deleted
  ));
END;

CREATE TRIGGER members_sync_update
AFTER UPDATE ON members
FOR EACH ROW
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('members', NEW.id, 'UPDATE', json_object(
    'id', NEW.id,
    'card_id', NEW.card_id,
    'short_card_id', NEW.short_card_id,
    'first_name', NEW.first_name,
    'last_name', NEW.last_name,
    'email', NEW.email,
    'phone', NEW.phone,
    'date_of_birth', NEW.date_of_birth,
    'auth_user_id', NEW.auth_user_id,
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at,
    'is_deleted', NEW.is_deleted
  ));
END;

CREATE TRIGGER members_sync_delete
AFTER DELETE ON members
FOR EACH ROW
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('members', OLD.id, 'DELETE', NULL);
END;
