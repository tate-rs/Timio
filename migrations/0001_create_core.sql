-- Projects table
CREATE TABLE IF NOT EXISTS projects (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  name          TEXT NOT NULL UNIQUE,
  note          TEXT
);

-- Time entries table
CREATE TABLE IF NOT EXISTS time_entries (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  started_at      INTEGER NOT NULL,                       -- UTC unix seconds
  ended_at        INTEGER,                                -- NULL while running
  description     TEXT,
  duration_seconds INTEGER
    GENERATED ALWAYS AS (
      CASE
        WHEN ended_at IS NULL THEN NULL
        ELSE ended_at - started_at
      END
    ) STORED,
  CHECK (ended_at IS NULL OR ended_at >= started_at)
);

-- View with “live” duration for running entries
-- strftime('%s','now') gives UTC unix seconds
CREATE VIEW IF NOT EXISTS v_time_entries AS
SELECT
  te.id,
  te.project_id,
  p.name AS project_name,
  te.started_at,
  te.ended_at,
  te.description,
  CASE
    WHEN te.ended_at IS NULL THEN (strftime('%s','now') - te.started_at)
    ELSE te.duration_seconds
  END AS effective_duration_seconds
FROM time_entries te
JOIN projects p ON p.id = te.project_id;
