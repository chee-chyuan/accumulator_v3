use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub struct AccumulatorFileStorage {
    pub master_file_path: String,
    pub epoch_file_path: String,
}

impl AccumulatorFileStorage {
    pub fn new(mas_file_path: String, epoch_file_path: String) -> AccumulatorFileStorage {
        AccumulatorFileStorage {
            master_file_path: mas_file_path,
            epoch_file_path: epoch_file_path,
        }
    }

    pub fn get_master_accumulator(&self, file_name: &str) -> Vec<u8> {
        let full_path = &self.concat_file_path(&self.master_file_path[..], file_name);
        self.read_from_file(full_path)
    }

    pub fn get_epoch_accumulator(&self, file_name: &str) -> Vec<u8> {
        let full_path = &self.concat_file_path(&self.epoch_file_path[..], file_name);
        self.read_from_file(full_path)
    }

    pub fn write_master_accumulator(&self, file_name: &str, content: &Vec<u8>) -> bool {
        let full_path = &self.concat_file_path(&self.master_file_path[..], file_name);
        self.write_to_file(full_path, content)
    }

    pub fn write_epoch_accumulator(&self, file_name: &str, content: &Vec<u8>) -> bool {
        let full_path = &self.concat_file_path(&self.epoch_file_path[..], file_name);
        self.write_to_file(full_path, content)
    }

    pub fn create_new_master_accumulator_file(&self, file_name: &str) -> bool {
        self.create_file_at_path(&self.master_file_path, file_name)
    }

    pub fn create_new_epoch_accumulator_file(&self, file_name: &str) -> bool {
        self.create_file_at_path(&self.epoch_file_path, file_name)
    }

    fn read_from_file(&self, path: &str) -> Vec<u8> {
        let mut file_content = Vec::new();
        let file_open_result = File::open(path);
        if let Err(_) = file_open_result  {
            return vec![];
        }
        let mut file = file_open_result.unwrap();
        
        file.read_to_end(&mut file_content).expect("Unable to read");
        file_content
    }

    fn write_to_file(&self, path: &str, content: &Vec<u8>) -> bool {
        let is_exist = std::path::Path::new(&path).exists();
        if !is_exist {
            return false;
        }

        let write_result = std::fs::write(path, content);
        let write_result = match write_result {
            Ok(_) => true,
            Err(_) => return false,
        };

        write_result
    }

    pub fn concat_file_path(&self, base_path: &str, file_name: &str) -> String {
        let mut full_path = String::new();
        full_path.push_str(base_path);
        full_path.push_str("/");
        full_path.push_str(file_name);

        full_path
    }

    fn create_file_at_path(&self, base_path: &str, file_name: &str) -> bool {
        let file_path = self.concat_file_path(&base_path, &file_name);
        let is_exist = std::path::Path::new(&file_path).exists();
        if is_exist {
            return false;
        }

        //will create a directory if it does not exist
        std::fs::create_dir_all(&base_path).unwrap();

        let empty_array: Vec<u8> = Vec::new();
        let write_result = std::fs::write(file_path, empty_array);
        let write_result = match write_result {
            Ok(_) => true,
            Err(_) => return false,
        };

        write_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::PathBuf;

    fn init_acculumulator_file_storage() -> AccumulatorFileStorage {
        let storage = AccumulatorFileStorage {
            master_file_path: String::from("path_master"),
            epoch_file_path: String::from("path_epoch"),
        };

        storage
    }

    fn create_file(base_path: &str, file_path: &str, content: &str) {
        let mut path = PathBuf::new();
        path.push(&file_path);
        std::fs::create_dir_all(&base_path).unwrap();
        std::fs::write(&path, content).unwrap();
    }

    fn delete_file(base_path: &str, file_path: &str) {
        std::fs::remove_file(&file_path).unwrap();
        std::fs::remove_dir(&base_path).unwrap();
    }

    fn create_file_assert_delete_file(
        base_path: &str,
        file_name: &str,
        f: impl Fn(&str) -> Vec<u8>,
    ) {
        let storage = init_acculumulator_file_storage();
        let text = "test to see if match";
        let file_path = storage.concat_file_path(&base_path, &file_name);

        create_file(&base_path, &file_path, &text);

        let file_content_byte = f(file_name);
        let file_content = std::str::from_utf8(&file_content_byte).unwrap();

        assert_eq!(text, file_content);
        delete_file(&base_path, &file_path);
    }

    #[test]
    fn concat_two_string_to_form_correct_file_path() {
        let storage = init_acculumulator_file_storage();

        let concat_text = storage.concat_file_path("base", "filename");
        assert_eq!(concat_text, "base/filename");
    }

    #[test]
    #[serial]
    fn return_master_accumulator_string_if_file_exist() {
        let storage = init_acculumulator_file_storage();
        let file_name = "test.txt";

        create_file_assert_delete_file(&storage.master_file_path, &file_name, |x| {
            storage.get_master_accumulator(x)
        })
    }

    #[test]
    fn return_master_accumulator_none_if_file_doest_not_exist() {
        let storage = init_acculumulator_file_storage();
        let storage_result = storage.get_master_accumulator("fileNotFound.txt");

        assert_eq!(&storage_result[..], []);
    }

    #[test]
    #[serial]
    fn creation_of_correct_master_accumulator_file() {
        let storage = init_acculumulator_file_storage();
        let file_name = "test_master_accumulator.txt";
        let full_path = storage.concat_file_path(&storage.master_file_path, &file_name);

        let is_exist = std::path::Path::new(&full_path).exists();
        assert_eq!(
            is_exist, false,
            "Ensure master accumulutor file is not created yet"
        );

        storage.create_new_master_accumulator_file(&file_name);
        let is_exist = std::path::Path::new(&full_path).exists();
        assert_eq!(is_exist, true, "Test if master accumulator file is created");

        delete_file(&storage.master_file_path, &full_path);
    }

    #[test]
    #[serial]
    fn creation_of_correct_epoch_accumulator_file() {
        let storage = init_acculumulator_file_storage();
        let file_name = "test_epoch_accumulator.txt";
        let full_path = storage.concat_file_path(&storage.epoch_file_path, &file_name);

        let is_exist = std::path::Path::new(&full_path).exists();
        assert_eq!(
            is_exist, false,
            "Ensure epoch accumulutor file is not created yet"
        );

        storage.create_new_epoch_accumulator_file(&file_name);
        let is_exist = std::path::Path::new(&full_path).exists();
        assert_eq!(is_exist, true, "Test if epoch accumulator file is created");

        delete_file(&storage.epoch_file_path, &full_path);
    }

    #[test]
    #[serial]
    fn return_epoch_accumulator_string_if_file_exist() {
        let storage = init_acculumulator_file_storage();
        let file_name = "test.txt";

        create_file_assert_delete_file(&storage.epoch_file_path, &file_name, |x| {
            storage.get_epoch_accumulator(x)
        })
    }

    #[test]
    fn return_epoch_accumulator_none_if_file_doest_not_exist() {
        let storage = init_acculumulator_file_storage();
        let storage_result = storage.get_epoch_accumulator("fileNotFound.txt");

        assert_eq!(storage_result, [].to_vec());
    }

    #[test]
    fn create_file_fail_when_file_exist() {
        let dir_path = "test_fail_file";
        let file_name = "text.txt";
        let file_path = "test_fail_file/text.txt";

        create_file(&dir_path, &file_path, "");

        let storage = init_acculumulator_file_storage();
        let result = storage.create_file_at_path(&dir_path, &file_name);

        assert_eq!(result, false);
        delete_file(&dir_path, &file_path);
    }

    #[test]
    fn create_file_successfully() {
        let dir_path = "test_success_file";
        let file_name = "text.txt";
        let file_path = "test_success_file/text.txt";

        let storage = init_acculumulator_file_storage();
        let result = storage.create_file_at_path(&dir_path, &file_name);

        assert_eq!(result, true, "Test file creation");

        let is_exist = std::path::Path::new(&file_path).exists();
        assert_eq!(is_exist, true, "Test file exist");

        delete_file(&dir_path, &file_path);
    }

    #[test]
    fn write_to_file_fail_if_file_does_not_exist() {
        let file_name = "this_file_does_not_exist.txt";
        let storage = init_acculumulator_file_storage();

        let random_array: Vec<u8> = vec![0, 1, 3];
        let result = storage.write_to_file(&file_name, &random_array);

        assert_eq!(result, false);
    }

    #[test]
    fn write_to_file_and_get_correct_value() {
        let file_name = "test_correct_value.txt";
        let base_path = ".";
        let storage = init_acculumulator_file_storage();

        let random_array: Vec<u8> = vec![0, 1, 3];
        create_file(&base_path, &file_name, "");

        let write_result = storage.write_to_file(&file_name, &random_array);
        assert_eq!(write_result, true);

        let vec_result = storage
            .read_from_file(&file_name);

        assert_eq!(random_array, vec_result);

        std::fs::remove_file(&file_name).unwrap();
    }
}
