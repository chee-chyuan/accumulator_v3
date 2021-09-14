pub mod epoch_sedes;
pub mod trie;

use epoch_sedes::EpochSede;
use trie::TrieTrait;

use ssz::{Decode, DecodeError, Encode};
use ssz_types::{typenum, VariableList};
use tree_hash::TreeHash;
use typenum::{U16777216, U2048};
use ethereum_types::{H256};

pub trait AccumulatorTrieTrait<T>
where
    T: TrieTrait,
{
    fn new(master_trie: T) -> Self;
    fn get_master_accumulator(&self) -> Result<Vec<H256>, DecodeError>;
    fn store_master_accumulator(&self, master_accumulator: &Vec<H256>) -> bool;
    fn get_epoch_accumulator_by_epoch_number(
        &self,
        epoch_number: &usize,
    ) -> Result<Vec<EpochSede>, DecodeError>;
    fn store_epoch_accumulator_by_epoch_number(
        &self,
        epoch_number: &usize,
        epoch_accumulator: &Vec<EpochSede>,
    ) -> bool;

    fn get_tree_root_hash(&self, epoch_accumulator: &Vec<EpochSede>) -> H256;
}

pub struct AccumulatorTrie<T: TrieTrait> {
    pub trie: T,
    pub epoch_size: u32,
}

impl<T> AccumulatorTrieTrait<T> for AccumulatorTrie<T>
where
    T: TrieTrait,
{
    fn new(trie: T) -> AccumulatorTrie<T> {
        AccumulatorTrie {
            trie,
            epoch_size: 2048,
        }
    }

    fn get_master_accumulator(&self) -> Result<Vec<H256>, DecodeError> {
        let encoded_ssz = self.trie.get_master_accumulator_encoded();
        let decoded = <VariableList<H256, U16777216>>::from_ssz_bytes(&encoded_ssz);

        match decoded {
            Ok(x) => return Ok(x.to_vec()),
            Err(x) => return Err(x),
        };
    }

    fn store_master_accumulator(&self, master_accumulator: &Vec<H256>) -> bool {
        let master_sede_var_list: VariableList<_, U16777216> =
            VariableList::from(master_accumulator.clone());

        let encoded_master_sede_var_list = master_sede_var_list.as_ssz_bytes();
        let result = self
            .trie
            .update_master_accumulator_encoded(&encoded_master_sede_var_list);

        result
    }

    //get and store functions for epoch_accumulator
    fn get_epoch_accumulator_by_epoch_number(
        &self,
        epoch_number: &usize,
    ) -> Result<Vec<EpochSede>, DecodeError> {
        let encoded_epoch = self.trie.get_epoch_accumulator_encoded(epoch_number);
        let epoch_sede_var_list_decoded =
            <VariableList<EpochSede, U2048>>::from_ssz_bytes(&encoded_epoch);

        match epoch_sede_var_list_decoded {
            Ok(x) => return Ok(x.to_vec()),
            Err(x) => return Err(x),
        };
    }

    fn store_epoch_accumulator_by_epoch_number(
        &self,
        epoch_number: &usize,
        epoch_accumulator: &Vec<EpochSede>,
    ) -> bool {
        let epoch_sede_fixed_vec: VariableList<_, U2048> =
            VariableList::from(epoch_accumulator.clone());
        let epoch_sede_var_list_encoded = &epoch_sede_fixed_vec.as_ssz_bytes();

        let result = self
            .trie
            .update_epoch_accumulator_encoded(epoch_number, epoch_sede_var_list_encoded);
        result
    }

    fn get_tree_root_hash(&self, epoch_accumulator: &Vec<EpochSede>) -> H256 {
        let epoch_sede_fixed_vec: VariableList<_, U2048> =
            VariableList::from(epoch_accumulator.clone());
        epoch_sede_fixed_vec.tree_hash_root()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accumulator_storage::accumulator_storage::AccumulatorFileStorage;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use tree_hash::TreeHash;
    use ethereum_types::U256;

    struct MockTrie {
        encoded_master_accumulator: RefCell<Vec<u8>>,
        encoded_epoch_accumulator: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl TrieTrait for MockTrie {
        fn new(_accumulator_storage: AccumulatorFileStorage) -> MockTrie {
            MockTrie {
                encoded_master_accumulator: RefCell::new(vec![]),
                encoded_epoch_accumulator: RefCell::new(HashMap::new()),
            }
        }

        fn get_master_accumulator_encoded(&self) -> Vec<u8> {
            self.encoded_master_accumulator.borrow().to_vec()
        }

        fn update_master_accumulator_encoded(&self, trie_encoded: &Vec<u8>) -> bool {
            let mut mut_tried_encoded = trie_encoded.clone();

            self.encoded_master_accumulator
                .borrow_mut()
                .append(&mut mut_tried_encoded);

            true
        }

        fn get_file_name_from_epoch_number(&self, epoch_number: &usize) -> String {
            epoch_number.to_string()
        }

        fn get_epoch_accumulator_encoded(&self, epoch_number: &usize) -> Vec<u8> {
            self.encoded_epoch_accumulator
                .borrow()
                .get(&epoch_number.to_string())
                .unwrap()
                .to_vec()
        }

        fn update_epoch_accumulator_encoded(
            &self,
            epoch_number: &usize,
            trie_encoded: &Vec<u8>,
        ) -> bool {
            self.encoded_epoch_accumulator
                .borrow_mut()
                .insert(epoch_number.to_string(), trie_encoded.to_vec());

            true
        }
    }

    #[test]
    fn get_correct_master_accumulator() {
        let test_vector: [u8; 4] = [1, 2, 5, 7];
        let hash = test_vector.tree_hash_root();

        let test_master_accumulator = vec![hash];
        let mut test_master_accumulator_encoded = test_master_accumulator.as_ssz_bytes();

        let accumulator_storage =
            AccumulatorFileStorage::new(String::from("not_used"), String::from("not_used"));
        let mock_master_trie = MockTrie::new(accumulator_storage);

        mock_master_trie
            .encoded_master_accumulator
            .borrow_mut()
            .append(&mut test_master_accumulator_encoded);

        let accumulator_trie = AccumulatorTrie::new(mock_master_trie);

        let master_accumulator = accumulator_trie.get_master_accumulator().unwrap();
        assert_eq!(test_master_accumulator, master_accumulator);
    }

    #[test]
    fn update_and_get_master_accumulator_correctly() {
        let test_vector: [u8; 4] = [1, 2, 5, 7];
        let hash = test_vector.tree_hash_root();

        let test_master_accumulator = vec![hash];

        let accumulator_storage =
            AccumulatorFileStorage::new(String::from("not_used"), String::from("not_used"));
        let mock_master_trie = MockTrie::new(accumulator_storage);

        let accumulator_trie = AccumulatorTrie::new(mock_master_trie);
        accumulator_trie.store_master_accumulator(&test_master_accumulator);
        //let test_master_accumulator_encoded = test_master_accumulator.as_ssz_bytes();
        let master_accumulator = accumulator_trie.get_master_accumulator().unwrap();

        assert_eq!(test_master_accumulator, master_accumulator);
    }

    #[test]
    fn get_correct_epoch_accumulator() {
        let epoch_sedes = EpochSede::new(U256::from(1234), H256::zero());

        let test_vector: Vec<EpochSede> = vec![epoch_sedes];
        let test_var_list: VariableList<_, U2048> = VariableList::from(test_vector.clone());
        let test_epoch_sede_var_list_encoded = &test_var_list.as_ssz_bytes();

        let accumulator_storage =
            AccumulatorFileStorage::new(String::from("not_used"), String::from("not_used"));
        let mock_trie = MockTrie::new(accumulator_storage);

        let epoch_number: usize = 12334;

        mock_trie.encoded_epoch_accumulator.borrow_mut().insert(
            epoch_number.to_string(),
            test_epoch_sede_var_list_encoded.to_vec(),
        );

        let accumulator_trie = AccumulatorTrie::new(mock_trie);
        let epoch_accumulator = accumulator_trie
            .get_epoch_accumulator_by_epoch_number(&epoch_number)
            .unwrap();

        assert_eq!(test_vector, epoch_accumulator);
    }

    #[test]
    fn update_and_get_epoch_accumulator_correctly() {
        let epoch_sedes = EpochSede::new(U256::from(123455), H256::zero());
        let test_vector: Vec<EpochSede> = vec![epoch_sedes];

        let accumulator_storage =
            AccumulatorFileStorage::new(String::from("not_used"), String::from("not_used"));
        let mock_master_trie = MockTrie::new(accumulator_storage);
        let accumulator_trie = AccumulatorTrie::new(mock_master_trie);

        let epoch_number: usize = 13043;
        accumulator_trie.store_epoch_accumulator_by_epoch_number(&epoch_number, &test_vector);

        let epoch_accumulator = accumulator_trie
            .get_epoch_accumulator_by_epoch_number(&epoch_number)
            .unwrap();
        assert_eq!(test_vector, epoch_accumulator);
    }

    #[test]
    fn test_get_correct_root_hash() {
        //how to test?
    }
}