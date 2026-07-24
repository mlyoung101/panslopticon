-- Ingress queue
CREATE TABLE IF NOT EXISTS ingress(
    id INTEGER PRIMARY KEY NOT NULL UNIQUE,
    url TEXT NOT NULL UNIQUE,
    date_added TEXT NOT NULL,
    origin_platform TEXT NOT NULL, -- i.e. github, reddit
    origin_src TEXT NOT NULL -- i.e. r/selfhosted; tag-llm
);

-- Confirmed slop
CREATE TABLE IF NOT EXISTS slop(
    id INTEGER PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    date_added TEXT NOT NULL,
    score REAL NOT NULL, -- why this was detected, the score
    panslop_version TEXT NOT NULL, -- version of panslopticon that detected this
    date_last_seen TEXT NOT NULL,
    dataset_path TEXT, -- Zstd compressed storage location on disk, once checked out
    origin_platform TEXT NOT NULL, -- i.e. github, reddit
    origin_src TEXT NOT NULL -- i.e. r/selfhosted; tag-llm
);

-- Considered before, but not slop; so we don't check things twice
CREATE TABLE IF NOT EXISTS not_slop(
    id INTEGER PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    date_added TEXT NOT NULL,
    score REAL NOT NULL
);

-- Github metrics
CREATE TABLE IF NOT EXISTS gh_metrics(
    slop_id INTEGER PRIMARY KEY NOT NULL,
    date TEXT NOT NULL,
    stars INTEGER NOT NULL,
    forks INTEGER NOT NULL,

    FOREIGN KEY (slop_id) REFERENCES slop(id)
);

-- List of detected agents in a repo
CREATE TABLE IF NOT EXISTS agents(
    slop_id INTEGER NOT NULL,
    agent TEXT NOT NULL,

    FOREIGN KEY (slop_id) REFERENCES slop(id)
);

-- Indices
CREATE INDEX IF NOT EXISTS not_slop_url_idx ON not_slop(url);
CREATE INDEX IF NOT EXISTS slop_url_idx ON slop(url);
