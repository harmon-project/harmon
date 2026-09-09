-- Add up migration script here

CREATE TABLE roles (
	id BLOB PRIMARY KEY NOT NULL,
	name TEXT NOT NULL,
	permissions INTEGER NOT NULL,
	created_at TEXT NOT NULL,
	updated_at TEXT NOT NULL
) STRICT;