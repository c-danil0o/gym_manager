-- Migration for Supabase sync functionality
-- Creates pending_changes table and triggers for tracking changes

CREATE TABLE IF NOT EXISTS pending_changes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  table_name TEXT NOT NULL,
  record_id INTEGER NOT NULL,
  operation TEXT NOT NULL CHECK (operation IN ('INSERT', 'UPDATE', 'DELETE')),
  record_data TEXT, -- JSON data for INSERT/UPDATE operations
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  retry_count INTEGER DEFAULT 0 NOT NULL,
  last_error TEXT
);

CREATE INDEX IF NOT EXISTS idx_pending_changes_created_at ON pending_changes (created_at);
CREATE INDEX IF NOT EXISTS idx_pending_changes_table_operation ON pending_changes (table_name, operation);

-- Trigger for members table
CREATE TRIGGER IF NOT EXISTS members_sync_insert 
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
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at,
    'is_deleted', NEW.is_deleted
  ));
END;

CREATE TRIGGER IF NOT EXISTS members_sync_update 
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
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at,
    'is_deleted', NEW.is_deleted
  ));
END;

CREATE TRIGGER IF NOT EXISTS members_sync_delete 
AFTER DELETE ON members 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('members', OLD.id, 'DELETE', NULL);
END;

-- Trigger for membership_types table
CREATE TRIGGER IF NOT EXISTS membership_types_sync_insert 
AFTER INSERT ON membership_types 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('membership_types', NEW.id, 'INSERT', json_object(
    'id', NEW.id,
    'name', NEW.name,
    'duration_days', NEW.duration_days,
    'visit_limit', NEW.visit_limit,
    'enter_by', NEW.enter_by,
    'price', NEW.price,
    'description', NEW.description,
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at,
    'is_deleted', NEW.is_deleted,
    'is_active', NEW.is_active
  ));
END;

CREATE TRIGGER IF NOT EXISTS membership_types_sync_update 
AFTER UPDATE ON membership_types 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('membership_types', NEW.id, 'UPDATE', json_object(
    'id', NEW.id,
    'name', NEW.name,
    'duration_days', NEW.duration_days,
    'visit_limit', NEW.visit_limit,
    'enter_by', NEW.enter_by,
    'price', NEW.price,
    'description', NEW.description,
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at,
    'is_deleted', NEW.is_deleted,
    'is_active', NEW.is_active
  ));
END;

CREATE TRIGGER IF NOT EXISTS membership_types_sync_delete 
AFTER DELETE ON membership_types 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('membership_types', OLD.id, 'DELETE', NULL);
END;

-- Trigger for memberships table
CREATE TRIGGER IF NOT EXISTS memberships_sync_insert 
AFTER INSERT ON memberships 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('memberships', NEW.id, 'INSERT', json_object(
    'id', NEW.id,
    'member_id', NEW.member_id,
    'membership_type_id', NEW.membership_type_id,
    'start_date', NEW.start_date,
    'end_date', NEW.end_date,
    'remaining_visits', NEW.remaining_visits,
    'status', NEW.status,
    'purchase_date', NEW.purchase_date,
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at,
    'is_deleted', NEW.is_deleted
  ));
END;

CREATE TRIGGER IF NOT EXISTS memberships_sync_update 
AFTER UPDATE ON memberships 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('memberships', NEW.id, 'UPDATE', json_object(
    'id', NEW.id,
    'member_id', NEW.member_id,
    'membership_type_id', NEW.membership_type_id,
    'start_date', NEW.start_date,
    'end_date', NEW.end_date,
    'remaining_visits', NEW.remaining_visits,
    'status', NEW.status,
    'purchase_date', NEW.purchase_date,
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at,
    'is_deleted', NEW.is_deleted
  ));
END;

CREATE TRIGGER IF NOT EXISTS memberships_sync_delete 
AFTER DELETE ON memberships 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('memberships', OLD.id, 'DELETE', NULL);
END;

-- Trigger for entry_logs table
CREATE TRIGGER IF NOT EXISTS entry_logs_sync_insert 
AFTER INSERT ON entry_logs 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('entry_logs', NEW.id, 'INSERT', json_object(
    'id', NEW.id,
    'member_id', NEW.member_id,
    'membership_id', NEW.membership_id,
    'entry_time', NEW.entry_time,
    'status', NEW.status,
    'created_at', NEW.created_at,
    'notes', NEW.notes
  ));
END;

CREATE TRIGGER IF NOT EXISTS entry_logs_sync_update 
AFTER UPDATE ON entry_logs 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('entry_logs', NEW.id, 'UPDATE', json_object(
    'id', NEW.id,
    'member_id', NEW.member_id,
    'membership_id', NEW.membership_id,
    'entry_time', NEW.entry_time,
    'status', NEW.status,
    'created_at', NEW.created_at,
    'notes', NEW.notes
  ));
END;

CREATE TRIGGER IF NOT EXISTS entry_logs_sync_delete 
AFTER DELETE ON entry_logs 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('entry_logs', OLD.id, 'DELETE', NULL);
END;

-- Trigger for users table
CREATE TRIGGER IF NOT EXISTS users_sync_insert 
AFTER INSERT ON users 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('users', NEW.id, 'INSERT', json_object(
    'id', NEW.id,
    'username', NEW.username,
    'password_hash', NEW.password_hash,
    'role', NEW.role,
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at
  ));
END;

CREATE TRIGGER IF NOT EXISTS users_sync_update 
AFTER UPDATE ON users 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('users', NEW.id, 'UPDATE', json_object(
    'id', NEW.id,
    'username', NEW.username,
    'password_hash', NEW.password_hash,
    'role', NEW.role,
    'created_at', NEW.created_at,
    'updated_at', NEW.updated_at
  ));
END;

CREATE TRIGGER IF NOT EXISTS users_sync_delete 
AFTER DELETE ON users 
FOR EACH ROW 
BEGIN
  INSERT INTO pending_changes (table_name, record_id, operation, record_data)
  VALUES ('users', OLD.id, 'DELETE', NULL);
END;