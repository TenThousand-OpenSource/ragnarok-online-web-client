-- Local development accounts only. Never use these on a server that is reachable from outside.
-- Passwords are stored as MD5 (login_conf.txt: use_MD5_passwords), the inter-server account's included.
INSERT INTO `login` (`account_id`, `userid`, `user_pass`, `sex`, `email`, `group_id`) VALUES (2000000, 'tester', MD5('tester'), 'M', 'tester@example.test', 99);
INSERT INTO `login` (`account_id`, `userid`, `user_pass`, `sex`, `email`, `group_id`) VALUES (2000001, 'tester2', MD5('tester2'), 'F', 'tester2@example.test', 0);
UPDATE `login` SET `user_pass` = MD5(`user_pass`) WHERE CHAR_LENGTH(`user_pass`) <> 32;
