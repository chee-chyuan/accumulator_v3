use accumulator_storage::accumulator_storage::AccumulatorFileStorage;
use accumulator_trie::trie::{Trie, TrieTrait};
use accumulator_trie::{AccumulatorTrie, AccumulatorTrieTrait};
use config::Config;
use ssz_types::{typenum, VariableList};
use tree_hash::TreeHash;
use typenum::U2048;

pub fn run(config: Config) {
    let accumulator_storage = AccumulatorFileStorage::new(
        config.master_accumulator_file_path,
        config.epoch_accumulator_file_path,
    );

    let trie = Trie::new(accumulator_storage);
    let accumulator_trie = AccumulatorTrie::new(trie);

    let master_accumulator = accumulator_trie.get_master_accumulator().unwrap();

    let mut is_all_hash_matched = true;
    for (i, epoch_hash) in master_accumulator.iter().enumerate() {
        let epoch_number = i + 1;
        let epoch_accumulator = accumulator_trie
            .get_epoch_accumulator_by_epoch_number(&epoch_number)
            .unwrap();
        let epoch_var_list: VariableList<_, U2048> = VariableList::from(epoch_accumulator.clone());

        println!(
            "{:?}- {:?}. Epoch length : {:?}",
            epoch_number,
            epoch_hash,
            epoch_var_list.len()
        );

        let epoch_hash_to_match = epoch_var_list.tree_hash_root();

        if epoch_hash_to_match != *epoch_hash {
            println!(
                "Hash does not match on Epoch Number: {:?}- {:?}",
                i, epoch_hash
            );
            is_all_hash_matched = false;
            break;
        }
    }

    if !is_all_hash_matched {
        println!("Hash does not match :(");
    } else {
        println!("All hash matched!");
    }
}
