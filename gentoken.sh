#!/bin/bash
date +%s%N | openssl md5 | awk '{print "INSERT INTO `geosub_tokens` (`token`) VALUES (\"" $2 "\"); SELECT `token` FROM `geosub_tokens` ORDER BY `id` DESC LIMIT 1;"};' | sqlite3 $1
