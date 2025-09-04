-- Drop existing tables if they exist (optional - only if you want to start fresh)
DROP TABLE IF EXISTS public.entry_logs;
DROP TABLE IF EXISTS public.memberships;
DROP TABLE IF EXISTS public.membership_types;
DROP TABLE IF EXISTS public.members;
DROP TABLE IF EXISTS public.users;

-- Create or update tables with correct structure
CREATE TABLE IF NOT EXISTS public.users (
  id BIGINT PRIMARY KEY,
  username TEXT,
  password_hash TEXT,
  role TEXT,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS public.members (
  id BIGINT PRIMARY KEY,
  card_id TEXT,
  short_card_id TEXT,
  first_name TEXT,
  last_name TEXT,
  email TEXT,
  phone TEXT,
  date_of_birth DATE,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  is_deleted BOOLEAN
);

CREATE TABLE IF NOT EXISTS public.membership_types (
  id BIGINT PRIMARY KEY,
  name TEXT,
  duration_days BIGINT,
  visit_limit BIGINT,
  enter_by BIGINT,
  price REAL,
  description TEXT,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  is_deleted BOOLEAN,
  is_active BOOLEAN
);

CREATE TABLE IF NOT EXISTS public.memberships (
  id BIGINT PRIMARY KEY,
  member_id BIGINT,
  membership_type_id BIGINT,
  start_date DATE,
  end_date DATE,
  remaining_visits BIGINT,
  status TEXT,
  purchase_date TIMESTAMPTZ,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  is_deleted BOOLEAN
);

-- Updated entry_logs table with correct structure
CREATE TABLE IF NOT EXISTS public.entry_logs (
  id BIGINT PRIMARY KEY,
  member_id BIGINT,
  membership_id BIGINT,
  card_id TEXT,
  member_name TEXT,
  entry_time TIMESTAMPTZ,
  local_date DATE,
  status TEXT,
  created_at TIMESTAMPTZ,
  notes TEXT
)
