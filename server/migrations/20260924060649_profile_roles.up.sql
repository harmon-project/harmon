-- Add up migration script here

CREATE TABLE profile_roles (
    id BLOB PRIMARY KEY NOT NULL,
    profile_id BLOB NOT NULL,
    role_id BLOB NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (profile_id) REFERENCES profiles (id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE,
    UNIQUE (profile_id, role_id)
) STRICT;