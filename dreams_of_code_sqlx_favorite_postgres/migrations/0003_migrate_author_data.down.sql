-- Re-add the author column to the book table
ALTER TABLE book
ADD COLUMN IF NOT EXISTS author TEXT;

-- Populate the author column with the concatenated names from authors
UPDATE book
SET author = a.name
FROM authors a
WHERE book.author_id = a.id;

-- Drop constraint on author
ALTER TABLE book
DROP CONSTRAINT fk_author;

-- Delete the inserted authors if they were created solely for this migration
DELETE FROM authors
WHERE id IN (
    SELECT DISTINCT author_id FROM book
);

-- Drop the foreign key constraint and the author_id column
ALTER TABLE book
DROP COLUMN author_id;
