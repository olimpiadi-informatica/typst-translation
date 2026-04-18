-- Add finalized_at to user_contest_status
ALTER TABLE user_contest_status ADD COLUMN finalized_at DATETIME;

-- Update existing finalized users to have a timestamp
UPDATE user_contest_status SET finalized_at = CURRENT_TIMESTAMP WHERE finalized_translations = TRUE;

-- Create contestant_print_status table
CREATE TABLE contestant_print_status (
    contestant_id INTEGER NOT NULL,
    contest_id INTEGER NOT NULL,
    printed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (contestant_id, contest_id),
    FOREIGN KEY (contestant_id) REFERENCES contestants(id),
    FOREIGN KEY (contest_id) REFERENCES contests(id)
);
