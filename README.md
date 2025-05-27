# Irys Rust SDK
SDK for interacting with Irys bundler nodes, using Rust.

## Examples
Code examples can be found in `examples` directory

## Client
For using the client binary, you have to build it using: 
```
cargo build --release --features="build-binary"
```

The client bin will be generated at `target/release/cli`. Then you can execute the binary with `./cli --help`

```
USAGE:
    cli <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    balance       Gets the specified user's balance for the current Irys bundler node
    fund          Funds your account with the specified amount of atomic units
    help          Print this message or the help of the given subcommand(s)
    price         Check how much of a specific token is required for an upload of <amount>
                      bytes
    upload        Uploads a specified file
    upload-dir    Uploads a folder (with a manifest)
    withdraw      Sends a fund withdrawal request
```
### Examples
```
./cli balance   <address>   --host <host> --token <token>
./cli price     <bytes>     --host <host> --token <token>
./cli fund      <amount>    --host <host> --token <token> --wallet <path | private_key>
./cli withdraw  <amount>    --host <host> --token <token> --wallet <path | private_key>
./cli upload    <file>      --host <host> --token <token> --wallet <path | private_key>
```

# Roadmap
Some functionalities are still work in progress. If you need to use one of them, you may want to have a look in the [js-sdk](https://github.com/Irys-xyz/js-sdk), or open an issue in this repository.
| Item            | Solana     | Ethereum  | ERC20     | Cosmos     | Aptos      |
|-----------------|------------|-----------|-----------|------------|------------|
| Balance         | [x]        | [x]       | [ ]       | [ ]        | [ ]        |
| Price           | [x]        | [x]       | [ ]       | [ ]        | [ ]        |
| Fund            | [ ]        | [ ]       | [ ]       | [ ]        | [ ]        |
| Withdraw        | [ ]        | [ ]       | [ ]       | [ ]        | [ ]        |
| Upload          | [x]        | [x]       | [ ]       | [ ]        | [ ]        |
| Upload Directory| [ ]        | [ ]       | [ ]       | [ ]        | [ ]        |
| Verify bundle   | [x]        | [x]       | [x]       | [x]        | [x]        |

# Testing
In order to run tests properly, you need to generate random bundles. Run:
```
npm install
npm run generate-bundles
```
To generate random bundles in `res/gen_bundles`, and then run:
```
cargo test
```