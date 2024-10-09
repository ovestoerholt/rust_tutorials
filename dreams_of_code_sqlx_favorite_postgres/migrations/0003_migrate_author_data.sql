ALTER TABLE book
ADD COLUMN author_id INT,
ADD CONSTRAINT fk_author
FOREIGN KEY (author_id) REFERENCES authors(id);

-- Insert unique authors into authors table
INSERT INTO authors (name)
SELECT DISTINCT author
FROM book;

-- Update the book table to point to the correct author_id
UPDATE book
SET author_id = a.id
FROM authors a
WHERE book.author = a.name;

-- Drop the old author column from book
ALTER TABLE book
DROP COLUMN IF EXISTS author;