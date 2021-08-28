CREATE TABLE history
(
    id BIGSERIAL PRIMARY KEY,
    data JSONB  NOT NULL,
    ts TIMESTAMP DEFAULT NOW()
);

CREATE TABLE fetchers
(
    id BIGSERIAL PRIMARY KEY,
    data JSONB  NOT NULL,
    ts TIMESTAMP DEFAULT NOW()
);