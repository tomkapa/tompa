-- Link each agent session to the qa_round it will contribute questions to
-- (used for parallel grooming where all roles write into the same round).
ALTER TABLE agent_sessions
    ADD COLUMN qa_round_id  UUID        REFERENCES qa_rounds(id),
    ADD COLUMN responded_at TIMESTAMPTZ;
