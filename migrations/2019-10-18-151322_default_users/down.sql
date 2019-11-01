DELETE FROM users WHERE user_id IN (0, 1, 2, 3, 4, 5);
DELETE FROM passwords WHERE user_id IN (0, 1, 2, 3, 4, 5);
ALTER SEQUENCE users_user_id_seq RESTART WITH 1;
