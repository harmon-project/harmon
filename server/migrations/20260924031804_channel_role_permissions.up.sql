-- Add up migration script here

CREATE TABLE channel_role_permissions (
    id BLOB PRIMARY KEY NOT NULL,
    channel_id BLOB NOT NULL,
    role_id BLOB NOT NULL,
    permissions INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (channel_id) REFERENCES channels (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
    UNIQUE (channel_id, role_id)
) STRICT;