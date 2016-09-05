CREATE TABLE IF NOT EXISTS GameServer (
    id                      SERIAL PRIMARY KEY,
    name                    VARCHAR NOT NULL,
    region                  VARCHAR NOT NULL,
    game_type               VARCHAR NOT NULL,
    ip                      VARCHAR NOT NULL,
);

CREATE TABLE IF NOT EXISTS Region (
	id			SERIAL PRIMARY KEY,
	name		VARCHAR NOT NULL UNIQUE
);

INSERT INTO Region (name) VALUES ('naeast') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('nawest') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('eueast') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('euwest') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('aswest') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('aseast') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('auwest') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('aueast') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('sanorth') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('sasouth') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('afnorth') ON CONFLICT (name) DO NOTHING;
INSERT INTO Region (name) VALUES ('afsouth') ON CONFLICT (name) DO NOTHING;

