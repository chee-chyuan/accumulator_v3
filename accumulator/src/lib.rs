use accumulator_storage::accumulator_storage::AccumulatorFileStorage;
use accumulator_trie::{
    epoch_sedes::EpochSede,
    trie::{Trie, TrieTrait},
    {AccumulatorTrie, AccumulatorTrieTrait},
};
use config::Config;
use db::block_db::{BlockDb, BlockDbTrait};
use ethereum_types::{H256, U256};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::{thread, time};

pub fn run(config: Config) {
    //start ctrl+c handler
    println!("Please press Ctrl + C to safely exit");
    let running = Arc::new(AtomicUsize::new(0));
    let r = running.clone();
    ctrlc::set_handler(move || {
        let prev = r.fetch_add(1, Ordering::SeqCst);
        if prev == 0 {
            println!("Exiting...");
        }
    })
    .expect("Error setting Ctrl-C handler");

    let accumulator_storage = AccumulatorFileStorage::new(
        config.master_accumulator_file_path,
        config.epoch_accumulator_file_path,
    );

    //init db and accumulator trie structs
    let block_db = BlockDb::new(config.block_connection_string);

    let trie = Trie::new(accumulator_storage);
    let accumulator_trie = AccumulatorTrie::new(trie);

    //get current master and epoch trie
    let mut master_accumulator = accumulator_trie.get_master_accumulator().unwrap();
    let mut epoch_number = master_accumulator.len();

    //if epoch number is 0, we can say that everything is suppose to be empty
    let mut current_epoch_accumulator: Vec<EpochSede> = vec![];
    if epoch_number > 0 {
        current_epoch_accumulator = accumulator_trie
            .get_epoch_accumulator_by_epoch_number(&epoch_number)
            .unwrap();

        //check hash to see if the loaded accumulators are correct
        let epoch_hash = accumulator_trie.get_tree_root_hash(&current_epoch_accumulator);
        let stored_hash = master_accumulator[epoch_number -1];

        //panic if invalid and advise user to clear all files
        if epoch_hash != stored_hash {
            panic!("Opps, hash mismatched. You may need to clear all files and try again");
        }
    } else {
        //lets read the file to create an empty epoch 1 file
        epoch_number = 1;
        accumulator_trie
            .get_epoch_accumulator_by_epoch_number(&epoch_number)
            .expect("something is wrong with the init epoch file");
    }

    let mut starting_block_number = get_starting_block_number(
        &config.starting_block_number,
        &(epoch_number as u32),
        &accumulator_trie.epoch_size,
        &(current_epoch_accumulator.len() as u32),
    );

    //perform action to append new blocks to the accumulator
    loop {

        let blocks = block_db.get_blocks(&(starting_block_number as u64), &500);

        if let None = blocks {
            println!("Loop completed, sleep for 5 seconds");
            let sleep_duration = time::Duration::from_millis(5000);
            thread::sleep(sleep_duration);
            continue;
        }

        for block in blocks.unwrap().iter() {
            if running.load(Ordering::SeqCst) > 0 {
                println!("Exited succesfully!");
                std::process::exit(0);
            }
            println!("Processing for block number: {}", block.block_number);

            let mut to_append_master_accumulator = false;

            if current_epoch_accumulator.len() as u32 == accumulator_trie.epoch_size {
                //epoch trie has been filled up
                //increment epoch number and create a new epoch trie

                println!("{:?}", current_epoch_accumulator.len());
                epoch_number = epoch_number + 1;
                current_epoch_accumulator = vec![];
                to_append_master_accumulator = true;

                println!("{:?}", epoch_number);
                println!("Yay new epoch!!!!!!!");
                // let sleep_duration = time::Duration::from_millis(15000);
                // thread::sleep(sleep_duration);

                //create a new epoch empty file at this current epoch number
                accumulator_trie
                    .get_epoch_accumulator_by_epoch_number(&epoch_number)
                    .expect("something is wrong with creating a new epoch file");
            }

            let block_number = block.block_number;
            let total_difficulty = U256::from_dec_str(&block.total_difficulty).unwrap();

            let serialized = impl_serde::serialize::from_hex(&block.block_hash).unwrap();
            let typed_block_hash = <H256>::from_slice(&serialized[..]);

            //get epoch_sede
            //append to current epoch_accumulator
            //get tree hash of epoch accumulator
            //and append or replace to master accumulator depending on whether
            //if this round starts with a new epoch trie
            let epoch_sede = EpochSede::new(total_difficulty, typed_block_hash);
            current_epoch_accumulator.push(epoch_sede);

            let epoch_hash = accumulator_trie.get_tree_root_hash(&current_epoch_accumulator);

            if !to_append_master_accumulator {
                //remove and insert to last element
                master_accumulator.pop();
            }
            master_accumulator.push(epoch_hash);

            //store both of them
            let store_epoch_result = accumulator_trie
                .store_epoch_accumulator_by_epoch_number(&epoch_number, &current_epoch_accumulator);
            let store_master_result =
                accumulator_trie.store_master_accumulator(&master_accumulator);

            println!(
                "Store epoch result : {:?}, epoch number : {:?}",
                store_epoch_result, epoch_number
            );
            println!("Store master result : {:?}", store_master_result);

            //we udpate the latest block number here
            starting_block_number = block_number + 1;
        }

        //loop end
        //sleep for 5 seconds to avoid putting too much load to the db in the next call
        println!("Loop completed, sleep for 0.5 seconds");
        let sleep_duration = time::Duration::from_millis(500);
        thread::sleep(sleep_duration);
    }
}

fn get_starting_block_number(
    &starting_block_number: &u32,
    epoch_number: &u32,
    epoch_size: &u32,
    current_epoch_accumulator_length: &u32,
) -> u32 {
    if *epoch_number == 0 as u32 {
        return starting_block_number;
    }

    starting_block_number + (epoch_number - 1) * epoch_size + current_epoch_accumulator_length
}

//when master and epoch are both empty
//start as usual

//when master is not empty but epoch is empty
//soemthing wrong

//when master is empty epoch is not empty
//something wrong

//when master and epoch are both not empty

#[cfg(test)]
mod tests {
    use db::models::block::Block;

    #[test]
    fn test_for_with_test_data() {
        let mut blocks: Vec<Block> = vec![];
        blocks.push(Block {
            block_number: 1234,
            block_hash: "0x12345".to_string(),
            total_difficulty: "1234553".to_string(),
        });
        blocks.push(Block {
            block_number: 12355,
            block_hash: "0x1232145".to_string(),
            total_difficulty: "1342334553".to_string(),
        });

        let blocks_options: Option<Vec<Block>> = Some(blocks);
        let mut i = 0;

        if let None = blocks_options {
            assert!(false);
        }

        for block in blocks_options.unwrap().iter() {
            println!("{:?}", block);
            i = i + 1;
        }

        assert_eq!(2, i);
    }
}
