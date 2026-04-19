-- Cleanup user_contest_status
ALTER TABLE user_contest_status DROP COLUMN envelope_received_at;
ALTER TABLE user_contest_status DROP COLUMN skip_envelope_verification;
