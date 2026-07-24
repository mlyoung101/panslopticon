CREATE TABLE full_text(
    slop_id INTEGER NOT NULL,
    file TEXT NOT NULL,
    text TEXT NOT NULL,

    FOREIGN KEY (slop_id) REFERENCES slop(id)
);
