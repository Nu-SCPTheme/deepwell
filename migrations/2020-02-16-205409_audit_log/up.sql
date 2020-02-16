CREATE TABLE audit_log (
    audit_log_entry_id BIGSERIAL PRIMARY KEY,
    audit_log_entry_type TEXT NOT NULL CHECK (
        audit_log_entry_type IN (
            'view_page',
            'add_page',
            'edit_page_content',
            'edit_page_tags',
            'remove_page'
            -- TODO
        )
    ),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    wiki_id BIGINT NOT NULL REFERENCES wikis(wiki_id),
    user_id BIGINT REFERENCES users(user_id),
    data JSONB NOT NULL
);

-- make table append-only
REVOKE UPDATE, DELETE, TRUNCATE ON TABLE audit_log FROM public;
