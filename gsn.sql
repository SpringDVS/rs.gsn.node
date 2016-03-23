BEGIN TRANSACTION;
CREATE TABLE "geosub_netspace" (
	`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
	`springname`	TEXT UNIQUE,
	`hostname`	TEXT,
	`address`	TEXT,
	`service`	INTEGER,
	`status`	INTEGER,
	`types`	INTEGER
);
CREATE TABLE "geosub_meta" (
	`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
	`settlement`	TEXT,
	`postcode`	TEXT,
	`county`	TEXT,
	`geosub`	TEXT
);
COMMIT;
