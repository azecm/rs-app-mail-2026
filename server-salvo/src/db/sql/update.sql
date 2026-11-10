

ALTER TABLE emails.boxes DROP COLUMN unread;
ALTER TABLE emails.boxes ADD source_file varchar(36) DEFAULT '' NOT NULL;
