-- Drop the old grooming_roles column and replace with full qa_config JSONB.
ALTER TABLE projects DROP COLUMN grooming_roles;

ALTER TABLE projects ADD COLUMN qa_config JSONB NOT NULL DEFAULT '{
  "grooming": {
    "business_analyst": {"model": "sonnet", "detail_level": 3, "max_questions": 3},
    "developer":        {"model": "sonnet", "detail_level": 3, "max_questions": 3},
    "ux_designer":      {"model": "sonnet", "detail_level": 3, "max_questions": 3},
    "security_engineer": {"model": "sonnet", "detail_level": 3, "max_questions": 3},
    "marketing":        {"model": "sonnet", "detail_level": 3, "max_questions": 3}
  },
  "planning":       {"model": "sonnet", "detail_level": 3, "max_questions": 3},
  "implementation": {"model": "sonnet", "detail_level": 2, "max_questions": 2}
}'::jsonb;
