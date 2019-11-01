-- Default or dummy users
INSERT INTO users (user_id, name, email, is_verified, is_bot, website, about, location)
    VALUES (
        0,
        'unknown',
        'unknown@example.com',
        true,
        true,
        'https://example.com/',
        'Standard account for unknown users',
        'unknown'
    );

INSERT INTO users (user_id, name, email, is_verified, is_bot, website, about, location)
    VALUES (
        1,
        'administrator',
        'noreply@example.com',
        true,
        true,
        'https://example.com/',
        'Standard account for root-level access',
        'Site-01'
    );

INSERT INTO users (user_id, name, email, is_verified, is_bot, website, about, location)
    VALUES (
        2,
        'system',
        'system@example.com',
        true,
        true,
        'https://example.com/',
        'Standard account for system actions',
        'everywhere'
    );

INSERT INTO users (user_id, name, email, is_verified, is_bot, website, about, location)
    VALUES (
        3,
        'anonymous',
        'anonymous@example.com',
        true,
        true,
        'https://example.com/',
        'Standard account for anonymous users',
        'unknown'
    );

INSERT INTO users (user_id, name, email, is_verified, is_bot, website, about, location)
    VALUES (
        4,
        'deleted',
        'deleted@example.com',
        true,
        true,
        'https://example.com/',
        'Standard account for deleted users',
        'deleted'
    );

INSERT INTO users (user_id, name, email, is_verified, is_bot, website, about, location)
    VALUES (
        5,
        'nobody',
        'nobody@example.com',
        true,
        true,
        'https://example.com/',
        'Standard account for unprivileged users',
        '?'
    );

-- Add invalid passwords to prevent logging in
INSERT INTO passwords (user_id, hash, salt, logn, param_r, param_p)
    VALUES (
        0,
        E'\\x0000000000000000000000000000000000000000000000000000000000000000',
        E'\\x00000000000000000000000000000000',
        1,
        1,
        1
    );

INSERT INTO passwords (user_id, hash, salt, logn, param_r, param_p)
    VALUES (
        1,
        E'\\x0000000000000000000000000000000000000000000000000000000000000000',
        E'\\x00000000000000000000000000000000',
        1,
        1,
        1
    );

INSERT INTO passwords (user_id, hash, salt, logn, param_r, param_p)
    VALUES (
        2,
        E'\\x0000000000000000000000000000000000000000000000000000000000000000',
        E'\\x00000000000000000000000000000000',
        1,
        1,
        1
    );

INSERT INTO passwords (user_id, hash, salt, logn, param_r, param_p)
    VALUES (
        3,
        E'\\x0000000000000000000000000000000000000000000000000000000000000000',
        E'\\x00000000000000000000000000000000',
        1,
        1,
        1
    );

INSERT INTO passwords (user_id, hash, salt, logn, param_r, param_p)
    VALUES (
        4,
        E'\\x0000000000000000000000000000000000000000000000000000000000000000',
        E'\\x00000000000000000000000000000000',
        1,
        1,
        1
    );

INSERT INTO passwords (user_id, hash, salt, logn, param_r, param_p)
    VALUES (
        5,
        E'\\x0000000000000000000000000000000000000000000000000000000000000000',
        E'\\x00000000000000000000000000000000',
        1,
        1,
        1
    );

-- Ensure new users don't overrun with existing users
ALTER SEQUENCE users_user_id_seq RESTART WITH 6;
