# accumulator_v3

To generate test vectors on the implementation of the block header accumulator in the portal network in [Trin](https://github.com/ethereum/trin).

## SSZ sedes
### Master accumulatr

`sede = List[epoch_hash:bytes32, 16777216]`

### Epoch accumulator

`sede = List[Container[block_hash:bytes32, total_difficulty:uint256],2048]`

## Setting up

create a config.json file that has the following structure:

```json
{
    "block_connection_string": "mysql://user:123456@localhost:3306/blockdb",
    "master_accumulator_file_path": "./accumulator_result/master",
    "epoch_accumulator_file_path":"./accumulator_result/epoch",
    "epoch_size": 2048,
    "starting_block_number": 0
}
```

change the block table name [here](https://github.com/chee-chyuan/accumulator_v3/blob/fea81cd99d7ccebd9f3264ea127639f9fb1c473b/db/src/block_db.rs#L48)


## Running the project

`cargo run -p accumulator`

To start generating the master and epoch accumulator

`cargo run -p accumulator_check_hash`

To check if the generated epoch accumulator has a matching hash to the stored master accumulator

To stop the program, use the `Ctrl+C` command to exit gracefully
