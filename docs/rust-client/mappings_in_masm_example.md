# Mapping

Example of smart contract that uses a mapping:
```masm
use.miden::account
use.std::sys

# Inputs: [KEY, VALUE]
# Outputs: []
export.write_to_map
  # The storage map is in storage slot 0
  push.0
  # => [index, KEY, VALUE]

  # Setting the key value pair in the map
  exec.account::set_map_item
  # => [OLD_MAP_ROOT, OLD_MAP_VALUE]

  dropw dropw dropw dropw
  # => []

  # Incrementing the nonce by 1
  push.1 exec.account::incr_nonce
  # => []
end

# Inputs: [KEY]
# Outputs: [VALUE]
export.get_value_in_map
  push.0
  # => [index]

  exec.account::get_map_item
  # => [VALUE]
end

# Inputs: []
# Outputs: [CURRENT_ROOT]
export.get_current_map_root
  # Getting the current root from slot 0
  push.0 exec.account::get_item
  # => [CURRENT_ROOT]

  exec.sys::truncate_stack
  # => [CURRENT_ROOT]
end
```

Transaction script that calls the smart contract above:

```masm
use.miden_by_example::mapping_example_contract
use.std::sys

begin
  push.1.2.3.4
  push.0.0.0.0
  # => [KEY, VALUE]

  call.mapping_example_contract::write_to_map
  # => []

  push.0.0.0.0
  # => [KEY]

  call.mapping_example_contract::get_value_in_map
  # => [VALUE]

  dropw
  # => []

  call.mapping_example_contract::get_current_map_root
  # => [CURRENT_ROOT]

  exec.sys::truncate_stack
end
```

### Running the example

To run the full example, navigate to the `rust-client` directory in the [miden-tutorials](https://github.com/0xPolygonMiden/miden-tutorials/) repository and run this command:

```bash
cd rust-client
cargo run --release --bin mapping_example
```