
		CREATE TABLE `geosub_netspace` (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT UNIQUE,
			`hostname`	TEXT,
			`address`	TEXT,
			`service`	INTEGER,
			`status`	INTEGER,
			`types`	INTEGER
		);
		
		CREATE TABLE `geotop_netspace` (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`springname`	TEXT,
			`hostname`	TEXT,
			`address`	TEXT,
			`service`	INTEGER,
			`priority`	INTEGER,
			`geosub`	TEXT
		);
		CREATE TABLE `geosub_tokens` (
			`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
			`token`	TEXT
		);

