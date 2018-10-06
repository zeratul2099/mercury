-- Your SQL goes here
CREATE TABLE `sensor_log2` (
  `sensor_id` int(11) NOT NULL DEFAULT '0',
  `sensor_name` varchar(128) DEFAULT NULL,
  `timestamp` datetime NOT NULL,
  `temperature` float DEFAULT NULL,
  `humidity` float DEFAULT NULL,
  PRIMARY KEY (`sensor_id`,`timestamp`)
) ENGINE=InnoDB DEFAULT CHARSET=latin1;
