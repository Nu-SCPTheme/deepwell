ALTER TABLE users
    ADD COLUMN email TEXT NOT NULL UNIQUE,
    ADD COLUMN author_page TEXT NOT NULL DEFAULT '',
    ADD COLUMN website TEXT NOT NULL DEFAULT '',
    ADD COLUMN about TEXT NOT NULL DEFAULT '',
    ADD COLUMN location TEXT NOT NULL DEFAULT '',
    ADD COLUMN gender TEXT NOT NULL DEFAULT '';

ALTER TABLE parents
    ADD COLUMN parented_by BIGINT NOT NULL REFERENCES users(user_id),
    ADD COLUMN parented_at TIMESTAMP NOT NULL;

CREATE TABLE passwords (
    user_id BIGINT PRIMARY KEY REFERENCES users(user_id),
    hash BYTEA NOT NULL,
    salt BYTEA NOT NULL,
    iterations INTEGER NOT NULL CHECK (iterations > 50000),
    key_size SMALLINT NOT NULL CHECK (key_size % 16 = 0),
    digest VARCHAR(12) NOT NULL CHECK (
        digest IN (
            'sha224',
            'sha256',
            'sha384',
            'sha512'
        )
    )
);

CREATE TABLE wikis (
    wiki_id BIGINT PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);

CREATE TABLE wiki_membership (
    wiki_id BIGINT NOT NULL REFERENCES wikis(wiki_id),
    user_id BIGINT NOT NULL REFERENCES users(user_id),
    applied_at TIMESTAMP NOT NULL,
    joined_at TIMESTAMP NOT NULL,
    PRIMARY KEY (wiki_id, user_id)
);

CREATE TABLE roles (
    role_id BIGINT PRIMARY KEY,
    wiki_id BIGINT NOT NULL REFERENCES wikis(wiki_id),
    name TEXT NOT NULL,
    permset BIT(20) NOT NULL,
    UNIQUE (wiki_id, name)
);

CREATE TABLE role_membership (
    wiki_id BIGINT REFERENCES wikis(wiki_id),
    role_id BIGINT REFERENCES roles(role_id),
    user_id BIGINT REFERENCES users(user_id),
    applied_at TIMESTAMP NOT NULL,
    PRIMARY KEY (wiki_id, role_Id, user_id)
);
