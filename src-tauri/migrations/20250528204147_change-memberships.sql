-- Add migration script here
-- Migration: Update memberships table structure
-- Change start_date and end_date from DATETIME to DATE
-- Update status CHECK constraint to new values

-- Step 1: Create a new table with the desired structure
CREATE TABLE memberships_new (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  member_id INTEGER NOT NULL,
  membership_type_id INTEGER NOT NULL,
  start_date DATE NOT NULL,
  end_date DATE,
  remaining_visits INTEGER,
  status TEXT NOT NULL CHECK (
    status IN (
      'active',
      'inactive',
      'expired',
      'suspended',
      'pending'
    )
  ),
  purchase_date DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
  FOREIGN KEY (member_id) REFERENCES members (id) ON DELETE CASCADE,
  FOREIGN KEY (membership_type_id) REFERENCES membership_types (id) ON DELETE RESTRICT
);

-- Step 2: Copy data from old table to new table, converting datetime to date
-- Also map old status values to new ones
INSERT INTO memberships_new (
  id,
  member_id,
  membership_type_id,
  start_date,
  end_date,
  remaining_visits,
  status,
  purchase_date,
  created_at,
  updated_at,
  is_deleted
)
SELECT
  id,
  member_id,
  membership_type_id,
  DATE(start_date) as start_date,
  DATE(end_date) as end_date,
  remaining_visits,
  CASE
    WHEN status = 'cancelled' THEN 'inactive'
    WHEN status = 'paused' THEN 'suspended'
    ELSE status  -- 'active', 'expired', 'pending' remain the same
  END as status,
  purchase_date,
  created_at,
  updated_at,
  is_deleted
FROM memberships;

-- Step 3: Drop the old table
DROP TABLE memberships;

-- Step 4: Rename the new table
ALTER TABLE memberships_new RENAME TO memberships;

-- Step 5: Recreate any indexes that existed on the original table (if any)
CREATE INDEX IF NOT EXISTS idx_membership_member_id ON memberships (member_id);

CREATE INDEX IF NOT EXISTS idx_membership_updated_at ON memberships (updated_at);
