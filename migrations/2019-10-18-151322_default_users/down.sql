DELETE FROM passwords WHERE user_id IN (0, 1, 2, 3, 4, 5, 6);
DELETE FROM authors WHERE user_id IN (0, 1, 2, 3, 4, 5, 6);
DELETE FROM ratings WHERE user_id IN (0, 1, 2, 3, 4, 5, 6);
DELETE FROM ratings_history WHERE user_id IN (0, 1, 2, 3, 4, 5, 6);
DELETE FROM users WHERE user_id IN (0, 1, 2, 3, 4, 5, 6);
ALTER SEQUENCE users_user_id_seq RESTART WITH 1;
