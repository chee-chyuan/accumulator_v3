use accumulator_storage::accumulator_storage::AccumulatorFileStorage;
pub trait TrieTrait  {
    fn new(accumulator_storage: AccumulatorFileStorage) -> Self;
    fn get_master_accumulator_encoded(&self) -> Vec<u8>;
    fn update_master_accumulator_encoded(&self, trie_encoded: &Vec<u8>) -> bool;
    fn get_file_name_from_epoch_number(&self, epoch_number: &usize) -> String;
    fn get_epoch_accumulator_encoded(&self, epoch_number: &usize) -> Vec<u8>;
    fn update_epoch_accumulator_encoded(
        &self,
        epoch_number: &usize,
        trie_encoded: &Vec<u8>,
    ) -> bool;
}

pub struct Trie {
    pub accumulator_storage: AccumulatorFileStorage,
    pub file_name: String,
}

impl TrieTrait for Trie {
    fn new(accumulator_storage: AccumulatorFileStorage) -> Trie {
        Trie {
            accumulator_storage,
            file_name: String::from("master_accumulator.txt"),
        }
    }

    fn get_master_accumulator_encoded(&self) -> Vec<u8> {
        //this will create the file if the file does not exist
        self.accumulator_storage
            .create_new_master_accumulator_file(&self.file_name);

        //find master accumulator from file
        let master_accumulator_file_content = self
            .accumulator_storage
            .get_master_accumulator(&self.file_name);

            master_accumulator_file_content
    }

    fn update_master_accumulator_encoded(&self, trie_encoded: &Vec<u8>) -> bool {
        let result = self
            .accumulator_storage
            .write_master_accumulator(&self.file_name, trie_encoded);

        result
    }

    fn get_file_name_from_epoch_number(&self, epoch_number: &usize) -> String {
        format!("epoch_accumulator_{}.txt", epoch_number)
    }

    fn get_epoch_accumulator_encoded(&self, epoch_number: &usize) -> Vec<u8> {
        //this will create the file if the file does not exist
        let file_name = self.get_file_name_from_epoch_number(epoch_number);
        self.accumulator_storage
            .create_new_epoch_accumulator_file(&file_name);

        //find epoch accumulator from file
        let epoch_accumulator_file_content = self
            .accumulator_storage
            .get_epoch_accumulator(&file_name);

        let byte_content = epoch_accumulator_file_content;
        byte_content
    }

    fn update_epoch_accumulator_encoded(&self, epoch_number: &usize, trie_encoded: &Vec<u8>) -> bool {
        let file_name = self.get_file_name_from_epoch_number(epoch_number);

        let result = self
            .accumulator_storage
            .write_epoch_accumulator(&file_name, trie_encoded);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use std::path::PathBuf;

    fn init_acculumulator_file_storage() -> AccumulatorFileStorage {
        let accumulator_storage = AccumulatorFileStorage::new(
            String::from("test_master_path"),
            String::from("test_epoch_path"),
        );
        accumulator_storage
    }

    fn init_trie(accumulator_storage: AccumulatorFileStorage) -> Trie {
        let trie = Trie::new(accumulator_storage);

        trie
    }

    fn concat_file_path(base_path: &str, file_name: &str) -> String {
        let mut full_path = String::new();
        full_path.push_str(base_path);
        full_path.push_str("/");
        full_path.push_str(file_name);

        full_path
    }

    fn create_file(base_path: &str, file_path: &str, content: &Vec<u8>) {
        let mut path = PathBuf::new();
        path.push(&file_path);
        std::fs::create_dir_all(&base_path).unwrap();
        std::fs::write(&path, content).unwrap();
    }

    fn delete_file(base_path: &str, file_path: &str) {
        fs::remove_file(&file_path).unwrap();
        fs::remove_dir(&base_path).unwrap();
    }

    #[test]
    #[serial]
    fn get_empty_array_when_master_trie_is_empty() {
        let accumulator_storage = init_acculumulator_file_storage();
        let base_path = String::from(&accumulator_storage.master_file_path);
        let master_trie = init_trie(accumulator_storage);

        let trie = master_trie.get_master_accumulator_encoded();
        let empty_vector: Vec<u8> = Vec::new();

        assert_eq!(empty_vector, trie);
        let file_path = concat_file_path(&base_path, &master_trie.file_name);

        delete_file(&base_path, &file_path)
    }

    #[test]
    #[serial]
    fn get_correct_array_from_master_trie() {
        let test_vec: Vec<u8> = vec![0, 1, 3, 4, 5];
        let accumulator_storage = init_acculumulator_file_storage();
        let base_path = String::from(&accumulator_storage.master_file_path);
        let master_trie = init_trie(accumulator_storage);

        let file_path = concat_file_path(&base_path, &master_trie.file_name);

        create_file(&base_path, &file_path, &test_vec);

        let vec_result = master_trie.get_master_accumulator_encoded();

        assert_eq!(test_vec, vec_result);

        delete_file(&base_path, &file_path);
    }

    #[test]
    #[serial]
    fn write_to_master_trie_successfully() {
        let test_vec: Vec<u8> = vec![0, 3, 5, 34, 5, 1];
        let empty_vec: Vec<u8> = Vec::new();
        let accumulator_storage = init_acculumulator_file_storage();
        let base_path = String::from(&accumulator_storage.master_file_path);
        let master_trie = init_trie(accumulator_storage);

        let file_path = concat_file_path(&base_path, &master_trie.file_name);

        create_file(&base_path, &file_path, &empty_vec);

        let update_result = master_trie.update_master_accumulator_encoded(&test_vec);
        assert_eq!(true, update_result);

        let vec_result = master_trie.get_master_accumulator_encoded();
        assert_eq!(test_vec, vec_result);

        delete_file(&base_path, &file_path);
    }

    #[test]
    #[serial]
    fn write_to_file_not_successfully_when_file_not_found() {
        let test_vec: Vec<u8> = vec![0, 3, 5, 34, 5, 1];
        let accumulator_storage = init_acculumulator_file_storage();
        let master_trie = init_trie(accumulator_storage);

        let update_result = master_trie.update_master_accumulator_encoded(&test_vec);
        assert_eq!(false, update_result);
    }
    

    #[test]
    fn able_to_generate_correct_epoch_file_name() {
        let accumulator_storage = init_acculumulator_file_storage();
        let epoch_trie = init_trie(accumulator_storage);

        let epoch_number: usize = 1234556789;
        let correct_file_name = "epoch_accumulator_1234556789.txt";

        let file_name = epoch_trie.get_file_name_from_epoch_number(&epoch_number);
        assert_eq!(correct_file_name, file_name);
    }
    

    #[test]
    #[serial]
    fn get_empty_array_when_epoch_trie_is_empty() {
        let accumulator_storage = init_acculumulator_file_storage();
        let base_path = String::from(&accumulator_storage.epoch_file_path);
        let epoch_trie = init_trie(accumulator_storage);

        let epoch_number = 134563743;
        let trie = epoch_trie.get_epoch_accumulator_encoded(&epoch_number);
        let empty_vector: Vec<u8> = Vec::new();

        assert_eq!(empty_vector, trie);

        let file_name = epoch_trie.get_file_name_from_epoch_number(&epoch_number);
        let file_path = concat_file_path(&base_path, &file_name);

        delete_file(&base_path, &file_path)
    }
}
