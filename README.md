# Bitcoin Scan Sighash

This tool is an educational source to learn more about Bitcoin's `SIGHASH_SINGLE` bug.
You can read the whole post [here](https://github.com/MatanHamilis/sighash_post)

## Requirements

To use this tool, you'll first have to run your own node of Bitcoin.
If you already have one, move to the next step.
Otherwise,  follow the instruction from  [Bitcoin core's website](https://bitcoincore.org/).
Notice it might take a while to synchronize your node.
In the end of the process your Bitcoin node should be up and running.
You'll also have to obtain the username and password for your RPC.

## Running

Use `cargo build --release` to build the program.
Nest, use `cargo run --release -- --help` to list all options available.
Currently three options are available:

1. `--address` - To specify the address of your bitcoin node, typically it listens to RPC commands on `http://127.0.0.1:8332`.
2. `--bitcoin-dir` - This is used to extract the credentials needed to access your bitcoin node.
3. `--log-file` - If specified the output will be written to the given log file, otherwise will be written to `stderr`.

**DISCLAIMER**: I haven't tested this on anything but Linux, so feel free to open issues.
