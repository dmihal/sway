contract;

use std::storage::StorageVec;

abi MyContract {
    #[storage(read, write)]
    fn push(value: b256);

    #[storage(read, write)]
    fn pop() -> b256;

    #[storage(read)]
    fn get(index: u64) -> b256;

    #[storage(read, write)]
    fn remove(index: u64) -> b256;

    #[storage(read, write)]
    fn swap_remove(index: u64) -> b256;

    #[storage(read, write)]
    fn set(index: u64, value: b256);

    #[storage(read, write)]
    fn insert(index: u64, value: b256);

    #[storage(read)]
    fn len() -> u64;

    #[storage(read)]
    fn is_empty() -> bool;

    #[storage(write)]
    fn clear();
}

storage {
    my_vec: StorageVec<b256> = StorageVec {},
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn push(value: b256) {
        storage.my_vec.push(value);
    }

    #[storage(read, write)]
    fn pop() -> b256 {
        storage.my_vec.pop().unwrap()
    }

    #[storage(read)]
    fn get(index: u64) -> b256 {
        storage.my_vec.get(index).unwrap()
    }

    #[storage(read, write)]
    fn remove(index: u64) -> b256 {
        storage.my_vec.remove(index)
    }

    #[storage(read, write)]
    fn swap_remove(index: u64) -> b256 {
        storage.my_vec.swap_remove(index)
    }

    #[storage(read, write)]
    fn set(index: u64, value: b256) {
        storage.my_vec.set(index, value);
    }

    #[storage(read, write)]
    fn insert(index: u64, value: b256) {
        storage.my_vec.insert(index, value);
    }

    #[storage(read)]
    fn len() -> u64 {
        storage.my_vec.len()
    }

    #[storage(read)]
    fn is_empty() -> bool {
        storage.my_vec.is_empty()
    }

    #[storage(write)]
    fn clear() {
        storage.my_vec.clear();
    }
}
