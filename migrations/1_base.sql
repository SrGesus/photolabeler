CREATE TABLE IF NOT EXISTS Directory (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL, 
  path TEXT NOT NULL UNIQUE
);

-- Temporary for development
INSERT INTO Directory (name, path)
VALUES ("images", "./images");

CREATE TABLE IF NOT EXISTS Image (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  directory_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  notes TEXT,
  UNIQUE (directory_id, name),
  FOREIGN KEY (directory_id)
    REFERENCES Directory (id)
    ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS Label (
  id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS Labeling (
  label_id INTEGER NOT NULL
    REFERENCES Label (id)
    ON DELETE CASCADE,
  image_id INTEGER NOT NULL
    REFERENCES Image (id)
    ON DELETE CASCADE,
  PRIMARY KEY (label_id, image_id)
);
