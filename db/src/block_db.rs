use super::models::block::Block;
use mysql::prelude::*;
use mysql::*;

pub trait BlockDbTrait {
    fn new(connection_string: String) -> BlockDb;
    fn get_blocks(&self, block_number_start: &u64, take: &u64) -> Option<Vec<Block>>;
}

#[derive(Debug)]
pub struct BlockDb {
    pub connection_string: String,
}

impl BlockDbTrait for BlockDb {
    fn new(connection_string: String) -> BlockDb {
        BlockDb {
            connection_string: connection_string,
        }
    }

    fn get_blocks(&self, block_number_start: &u64, take: &u64) -> Option<Vec<Block>> {
        let opts = Opts::from_url(&self.connection_string);
        if let Err(x) = opts {
            print!("{:?}", x);
            return None;
        }

        let opts = opts.unwrap();

        let pool = Pool::new(opts);
        if let Err(x) = pool {
            print!("{:?}", x);
            return None;
        }

        let pool = pool.unwrap();

        let conn = pool.get_conn();
        if let Err(x) = conn {
            print!("{:?}", x);
            return None;
        }

        let mut conn = conn.unwrap();

        let blocks = conn.exec_map(
            "SELECT number, hash, totaldifficulty from blocks where number >= :number limit :take",
            params! { "number" => &block_number_start, "take" => &take},
            |(number, hash, totaldifficulty)| Block {
                block_number: number,
                block_hash: hash,
                total_difficulty: totaldifficulty,
            }
        );

        let blocks = match blocks {
            Ok(blocks) => blocks,
            Err(_) => return None,
        };

        Some(blocks)
    }
}
