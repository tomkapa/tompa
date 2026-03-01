CREATE TABLE task_dependencies (
    id                  uuid        PRIMARY KEY,
    task_id             uuid        NOT NULL REFERENCES tasks (id),
    depends_on_task_id  uuid        NOT NULL REFERENCES tasks (id),
    created_at          timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT uq_task_dependencies UNIQUE (task_id, depends_on_task_id),
    CONSTRAINT chk_task_dependencies_no_self CHECK (task_id != depends_on_task_id)
);
