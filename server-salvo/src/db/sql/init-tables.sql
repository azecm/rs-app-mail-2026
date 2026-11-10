

CREATE TABLE emails.boxes
(
    idb         bigserial                     NOT NULL,
    idu         int4                          NOT NULL,
    box         int4                          NOT NULL,
    "date"      timestamp DEFAULT now()       NOT NULL,
    unread      bool      DEFAULT false       NOT NULL,
    sender      jsonb     DEFAULT '{}'::jsonb NOT NULL,
    recipient   jsonb     DEFAULT '{}'::jsonb NOT NULL,
    subject     text      DEFAULT ''::text    NOT NULL,
    "content"   text      DEFAULT ''::text    NOT NULL,
    attachments jsonb                         NULL,
    CONSTRAINT boxes_pkey PRIMARY KEY (idb)
);

CREATE TABLE emails.notes
(
    idn        serial4                                   NOT NULL,
    idp        int4                                      NOT NULL,
    idu        int4                                      NOT NULL,
    "position" int4                                      NOT NULL,
    "label"    varchar(50) DEFAULT ''::character varying NOT NULL,
    email      varchar(50) DEFAULT ''::character varying NOT NULL,
    "content"  text        DEFAULT ''::text              NOT NULL,
    "event"    jsonb                                     NULL,
    CONSTRAINT notes_pkey PRIMARY KEY (idn)
);

CREATE TABLE emails.users
(
    idu        serial4                                   NOT NULL,
    "name"     varchar(50) DEFAULT ''::character varying NOT NULL,
    "password" varchar(50) DEFAULT ''::character varying NOT NULL,
    email      varchar(50) DEFAULT ''::character varying NOT NULL,
    signature  text        DEFAULT ''::text              NOT NULL,
    CONSTRAINT users_pkey PRIMARY KEY (idu)
);
