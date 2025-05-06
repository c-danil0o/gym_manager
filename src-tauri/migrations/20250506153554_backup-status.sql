CREATE TABLE IF NOT EXISTS backup_status (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  last_check_time DATETIME NOT NULL, -- When the check was performed
  last_successful_upload_time DATETIME, -- Timestamp of the *data* successfully uploaded
  status TEXT NOT NULL CHECK (
    status IN (
      'checked_no_changes',
      'upload_success',
      'upload_failed'
    )
  ),
  error_message TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_backup_status_timestamp ON backup_status (last_successful_upload_time);

CREATE INDEX IF NOT EXISTS idx_backup_status_created_at ON backup_status (created_at);
