CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    pwd VARCHAR(50) NOT NULL
);

INSERT INTO users (username, pwd) VALUES
('mock1', 'password1'),
('mock2', 'password2'),
('mock3', 'password3'),
('admin', 'admin');
