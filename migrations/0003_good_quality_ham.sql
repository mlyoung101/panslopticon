-- Genuine, confirmed, specifically sought out good quality (non AI) data
-- Used for training the "ham" dataset in the naive Bayes classifier
CREATE TABLE IF NOT EXISTS ham(
    id INTEGER PRIMARY KEY NOT NULL,
    url TEXT NOT NULL,
    date_added TEXT NOT NULL,
    score REAL NOT NULL
);

CREATE TABLE IF NOT EXISTS ham_full_text(
    id INTEGER NOT NULL,
    file TEXT NOT NULL,
    text TEXT NOT NULL,

    FOREIGN KEY (id) REFERENCES ham(id)
);
