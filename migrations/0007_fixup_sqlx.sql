-- SQLx is apparently the worst framework of all time and simply explodes if it ever sees a null ever
-- So we have to make this not null
ALTER TABLE ham DROP COLUMN date_last_seen;
ALTER TABLE ham ADD COLUMN date_last_seen STRING NOT NULL DEFAULT "";
UPDATE ham SET date_last_seen = date_added;
