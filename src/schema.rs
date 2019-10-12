table! {
    authors (page_id, user_id, author_type) {
        page_id -> Int8,
        user_id -> Int8,
        author_type -> Text,
        created_at -> Date,
    }
}

table! {
    files (file_id) {
        file_id -> Int8,
        file_name -> Text,
        file_uri -> Text,
        description -> Text,
        page_id -> Int8,
    }
}

table! {
    pages (page_id) {
        page_id -> Int8,
        created_at -> Timestamp,
        slug -> Text,
        title -> Text,
        alt_title -> Nullable<Text>,
        tags -> Array<Text>,
    }
}

table! {
    parents (page_id, parent_page_id) {
        page_id -> Int8,
        parent_page_id -> Int8,
        parented_by -> Int8,
        parented_at -> Timestamp,
    }
}

table! {
    passwords (user_id) {
        user_id -> Int8,
        hash -> Bytea,
        salt -> Bytea,
        iterations -> Int4,
        key_size -> Int2,
        digest -> Varchar,
    }
}

table! {
    ratings (page_id, user_id) {
        page_id -> Int8,
        user_id -> Int8,
        rating -> Int2,
    }
}

table! {
    ratings_history (rating_id) {
        rating_id -> Int8,
        page_id -> Int8,
        user_id -> Int8,
        created_at -> Timestamp,
        rating -> Nullable<Int2>,
    }
}

table! {
    revisions (revision_id) {
        revision_id -> Int8,
        created_at -> Timestamp,
        page_id -> Int8,
        user_id -> Int8,
        git_commit -> Text,
        changes -> Jsonb,
    }
}

table! {
    role_membership (wiki_id, role_id, user_id) {
        wiki_id -> Int8,
        role_id -> Int8,
        user_id -> Int8,
        applied_at -> Timestamp,
    }
}

table! {
    roles (role_id) {
        role_id -> Int8,
        wiki_id -> Int8,
        name -> Text,
        permset -> Bit,
    }
}

table! {
    users (user_id) {
        user_id -> Int8,
        name -> Text,
        created_at -> Timestamp,
        email -> Text,
        author_page -> Text,
        website -> Text,
        about -> Text,
        location -> Text,
        gender -> Text,
    }
}

table! {
    wiki_membership (wiki_id, user_id) {
        wiki_id -> Int8,
        user_id -> Int8,
        applied_at -> Timestamp,
        joined_at -> Timestamp,
    }
}

table! {
    wikis (wiki_id) {
        wiki_id -> Int8,
        slug -> Text,
        name -> Text,
        created_at -> Timestamp,
    }
}

joinable!(authors -> pages (page_id));
joinable!(authors -> users (user_id));
joinable!(files -> pages (page_id));
joinable!(parents -> users (parented_by));
joinable!(passwords -> users (user_id));
joinable!(ratings_history -> pages (page_id));
joinable!(ratings_history -> users (user_id));
joinable!(revisions -> pages (page_id));
joinable!(revisions -> users (user_id));
joinable!(role_membership -> roles (role_id));
joinable!(role_membership -> users (user_id));
joinable!(role_membership -> wikis (wiki_id));
joinable!(roles -> wikis (wiki_id));
joinable!(wiki_membership -> users (user_id));
joinable!(wiki_membership -> wikis (wiki_id));

allow_tables_to_appear_in_same_query!(
    authors,
    files,
    pages,
    parents,
    passwords,
    ratings,
    ratings_history,
    revisions,
    role_membership,
    roles,
    users,
    wiki_membership,
    wikis,
);
