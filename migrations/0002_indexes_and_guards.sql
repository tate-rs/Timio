-- One open timer per project
CREATE UNIQUE INDEX IF NOT EXISTS idx_one_open_timer_per_project
  ON time_entries(project_id)
  WHERE ended_at IS NULL;

-- Common lookups
CREATE INDEX IF NOT EXISTS idx_time_entries_project_started
  ON time_entries(project_id, started_at);

-- Speed up filtering finished entries
CREATE INDEX IF NOT EXISTS idx_time_entries_ended
  ON time_entries(ended_at);
