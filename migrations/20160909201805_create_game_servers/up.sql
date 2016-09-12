CREATE TABLE game_servers (
    id                      SERIAL PRIMARY KEY,
    name                    VARCHAR NOT NULL,
    region                  VARCHAR NOT NULL,
    game_type               VARCHAR NOT NULL,
    ip                      VARCHAR NOT NULL,
    max_users               INT NOT NULL,
    current_users           INT NOT NULL DEFAULT 0,
    current_premium_users   INT DEFAULT 0,
    max_premium_users       INT,
    tags                    VARCHAR[] NOT NULL
)
