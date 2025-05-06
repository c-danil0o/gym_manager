CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS members (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  card_id TEXT UNIQUE,
  short_card_id TEXT UNIQUE,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  email TEXT UNIQUE,
  phone TEXT,
  date_of_birth DATE,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  is_deleted BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_member_card_id ON members (card_id);

CREATE INDEX IF NOT EXISTS idx_short_member_card_id ON members (short_card_id);

CREATE INDEX IF NOT EXISTS idx_member_updated_at ON members (updated_at);

CREATE TABLE IF NOT EXISTS membership_types (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  duration_days INTEGER,
  visit_limit INTEGER,
  price REAL NOT NULL,
  enter_by INTEGER,
  description TEXT,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  is_deleted BOOLEAN DEFAULT FALSE NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_membership_type_updated_at ON membership_types (updated_at);

CREATE TABLE IF NOT EXISTS memberships (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  member_id INTEGER NOT NULL,
  membership_type_id INTEGER NOT NULL,
  start_date DATETIME NOT NULL,
  end_date DATETIME,
  remaining_visits INTEGER,
  status TEXT NOT NULL CHECK (
    status IN (
      'active',
      'expired',
      'cancelled',
      'pending',
      'paused'
    )
  ),
  purchase_date DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  is_deleted BOOLEAN DEFAULT FALSE NOT NULL,
  FOREIGN KEY (member_id) REFERENCES members (id) ON DELETE CASCADE,
  FOREIGN KEY (membership_type_id) REFERENCES membership_types (id) ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS idx_membership_member_id ON memberships (member_id);

CREATE INDEX IF NOT EXISTS idx_membership_updated_at ON memberships (updated_at);

CREATE TABLE IF NOT EXISTS entry_logs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  member_id INTEGER NOT NULL,
  membership_id INTEGER,
  entry_time DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  status TEXT NOT NULL CHECK (
    status IN (
      'allowed',
      'denied_expired',
      'denied_no_membership',
      'denied_no_visits',
      'denied_frozen'
    )
  ),
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
  FOREIGN KEY (member_id) REFERENCES members (id) ON DELETE CASCADE,
  FOREIGN KEY (membership_id) REFERENCES memberships (id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_entry_log_entry_time ON entry_logs (entry_time);
