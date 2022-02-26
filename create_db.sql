; Create the database
DROP TABLE IF EXISTS debug_ids;
CREATE TABLE debug_ids (
    debug_id CHAR(36) PRIMARY KEY not NULL,
    location VARCHAR(750),
    note VARCHAR(750)
);  
