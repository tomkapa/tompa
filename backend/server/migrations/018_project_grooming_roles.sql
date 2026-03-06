-- Add configurable grooming roles to projects.
-- Defaults to all 5 roles enabled. Business analyst is always required (enforced at app layer).
ALTER TABLE projects
ADD COLUMN grooming_roles text[] NOT NULL
    DEFAULT ARRAY['business_analyst','developer','ux_designer','security_engineer','marketing'];
