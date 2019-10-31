-- Account info

CREATE TABLE users (
    user_id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_bot BOOLEAN NOT NULL DEFAULT false,
    author_page TEXT NOT NULL DEFAULT '',
    website TEXT NOT NULL DEFAULT '',
    about TEXT NOT NULL DEFAULT '',
    gender TEXT NOT NULL DEFAULT '' CHECK (gender = LOWER(gender)),
    location TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE passwords (
    user_id BIGSERIAL PRIMARY KEY REFERENCES users(user_id),
    hash BYTEA NOT NULL CHECK (LENGTH(hash) * 8 = 256),
    salt BYTEA NOT NULL CHECK (LENGTH(salt) * 8 = 128),
    logn SMALLINT NOT NULL CHECK (ABS(logn) < 128),
    param_r INTEGER NOT NULL,
    param_p INTEGER NOT NULL
);

-- Wikis and wiki settings

CREATE TABLE wikis (
    wiki_id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    domain TEXT NOT NULL UNIQUE CHECK(domain = LOWER(domain)),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE wiki_membership (
    wiki_id BIGSERIAL NOT NULL REFERENCES wikis(wiki_id),
    user_id BIGSERIAL NOT NULL REFERENCES users(user_id),
    applied_at TIMESTAMP WITH TIME ZONE NOT NULL,
    joined_at TIMESTAMP WITH TIME ZONE NOT NULL,
    banned_at TIMESTAMP WITH TIME ZONE, -- null = not banned
    banned_until TIMESTAMP WITH TIME ZONE, -- null = indefinite ban
    PRIMARY KEY (wiki_id, user_id)
);

CREATE TABLE roles (
    role_id BIGSERIAL PRIMARY KEY,
    wiki_id BIGSERIAL NOT NULL REFERENCES wikis(wiki_id),
    name TEXT NOT NULL,
    permset JSONB NOT NULL,
    UNIQUE (wiki_id, name)
);

CREATE TABLE role_membership (
    wiki_id BIGSERIAL REFERENCES wikis(wiki_id),
    role_id BIGSERIAL REFERENCES roles(role_id),
    user_id BIGSERIAL REFERENCES users(user_id),
    applied_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (wiki_id, role_Id, user_id)
);

-- Pages and revisions

CREATE TABLE pages (
    page_id BIGSERIAL PRIMARY KEY,
    wiki_id BIGSERIAL NOT NULL REFERENCES wikis(wiki_id),
    slug TEXT NOT NULL,
    title TEXT NOT NULL,
    alt_title TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    UNIQUE (deleted_at, slug)
);

CREATE TABLE parents (
    page_id BIGSERIAL NOT NULL REFERENCES pages(page_id),
    parent_page_id BIGSERIAL NOT NULL REFERENCES pages(page_id),
    parented_by BIGSERIAL NOT NULL REFERENCES users(user_id),
    parented_at TIMESTAMP WITH TIME ZONE NOT NULL,
    PRIMARY KEY (page_id, parent_page_id)
);

CREATE TABLE revisions (
    revision_id BIGSERIAL PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    page_id BIGSERIAL NOT NULL REFERENCES pages(page_id),
    user_id BIGSERIAL NOT NULL REFERENCES users(user_id),
    message TEXT NOT NULL,
    git_commit BIT(160) NOT NULL UNIQUE,
    change_type VARCHAR(8) NOT NULL CHECK (
        change_type IN (
            'create',
            'modify',
            'delete',
            'rename',
            'tags'
        )
    )
);

CREATE TABLE tag_history (
    revision_id BIGSERIAL REFERENCES revisions(revision_id) PRIMARY KEY,
    added_tags TEXT[] NOT NULL,
    removed_tags TEXT[] NOT NULL,
    CHECK (NOT(added_tags && removed_tags))
);

CREATE TABLE ratings (
    page_id BIGSERIAL NOT NULL,
    user_id BIGSERIAL NOT NULL,
    rating SMALLINT NOT NULL,
    PRIMARY KEY (page_id, user_id)
);

CREATE TABLE ratings_history (
    rating_id BIGSERIAL PRIMARY KEY,
    page_id BIGSERIAL NOT NULL REFERENCES pages(page_id),
    user_id BIGSERIAL NOT NULL REFERENCES users(user_id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    rating SMALLINT
);

CREATE TABLE authors (
    page_id BIGSERIAL NOT NULL REFERENCES pages(page_id),
    user_id BIGSERIAL NOT NULL REFERENCES users(user_id),
    author_type TEXT NOT NULL CHECK (
        author_type IN (
            'author',
            'rewrite',
            'translator',
            'maintainer'
        )
    ),
    written_at DATE NOT NULL,
    PRIMARY KEY (page_id, user_id, author_type)
);

-- Hosted files

CREATE TABLE files (
    file_id BIGSERIAL PRIMARY KEY,
    file_name TEXT NOT NULL UNIQUE,
    file_uri TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    page_id BIGSERIAL NOT NULL REFERENCES pages(page_id)
);
