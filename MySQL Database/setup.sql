CREATE DATABASE bitcoin_data;
SHOW DATABASES;
USE bitcoin_data;
CREATE TABLE blocks (
    id INT AUTO_INCREMENT PRIMARY KEY,
    block_height BIGINT NOT NULL,
    timestamp DATETIME NOT NULL
);


select * from blocks;
DROP TABLE IF EXISTS blocks;

ALTER TABLE blocks MODIFY COLUMN block_height VARCHAR(66);
ALTER TABLE blocks
ADD COLUMN transaction_count INT NOT NULL,
ADD COLUMN block_size BIGINT NOT NULL,
ADD COLUMN fees_collected DECIMAL(16, 8) NOT NULL,
ADD COLUMN miner_address VARCHAR(42) NOT NULL,
ADD COLUMN avg_transaction_size BIGINT NOT NULL;

select * from blocks;

