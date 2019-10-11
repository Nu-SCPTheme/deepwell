ALTER TABLE users
    DROP COLUMN email,
    DROP COLUMN author_page,
    DROP COLUMN website,
    DROP COLUMN about,
    DROP COLUMN location,
    DROP COLUMN gender;

ALTER TABLE parents
    DROP COLUMN parented_by,
    DROP COLUMN parented_at;

DROP TABLE passwords;
DROP TABLE role_membership;
DROP TABLE roles;
DROP TABLE wiki_membership;
DROP TABLE wikis;
