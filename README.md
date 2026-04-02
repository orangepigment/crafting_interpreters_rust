# Crafting Interpreters in Rust
A Rust implementation of the interpreter from ["Crafting Interpreters"](https://craftinginterpreters.com) by Robert Nystrom.

## Testing
The implementation is tested with the help of tools from the original books repo. See [the instructions](https://github.com/munificent/craftinginterpreters?tab=readme-ov-file#testing-your-implementation) for setup.
For integrations tests to work properly you need to define the following environment variables:
- TEST_TOOLS_LOCATION=`<path to the root folder of the craftinginterpreters repo>`
- SUITE_NAME=`<test suite name from the craftinginterpreters repo>`

To run tests use:
```sh
export TEST_TOOLS_LOCATION="<path>/craftinginterpreters"
export SUITE_NAME="<suite_name>"

cargo test
```