use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use bitcoincore_rpc::bitcoin::blockdata::script::Instruction;
use bitcoincore_rpc::bitcoin::{Address, SigHashType, TxOut, Txid};
use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::{Auth, Client};
use log::{error, info, LevelFilter};
use simple_logging::{log_to_file, log_to_stderr};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "sighashargs", about = "Bitcoin SigHash Scanner Arguments")]
struct ProgramArguments {
    user: String,
    pass: String,
    #[structopt(default_value = "127.0.0.1:8332")]
    address: SocketAddr,
    #[structopt(parse(from_os_str))]
    log_file: Option<PathBuf>,
}

fn main() {
    let args = ProgramArguments::from_args();
    let mut url = "http://".to_string();
    url.push_str(args.address.to_string().as_str());

    let user = args.user;
    let pass = args.pass;
    match args.log_file {
        None => log_to_stderr(LevelFilter::Info),
        Some(f) => log_to_file(
            f.to_str()
                .expect("Can't convert given log path to string, leaving!"),
            LevelFilter::Info,
        )
        .expect("Failed to set up logging!"),
    }

    const MAX_BLOCK_HEIGHT: u64 = 710000;
    let auth = Auth::UserPass(user, pass);
    let client = Client::new(url.as_str(), auth).unwrap();
    let mut utxos = HashMap::<(Txid, usize), TxOut>::new();
    (0..MAX_BLOCK_HEIGHT).into_iter().for_each(|height| {
        if height % 500 == 0 {
            info!("height: {}", height);
        }
        let current_block_hash = match client.get_block_hash(height) {
            Err(_) => {
                error!("leaving, error getting blockhash for height: {} ", height);
                error!("Failed to get block!");
                error!("Is bitcoin daemon running?");
                panic!();
            }
            Ok(h) => h,
        };

        let current_block = client.get_block(&current_block_hash).unwrap();
        current_block.txdata.clone().iter().for_each(|tx| {
            for (vout, txout) in tx.output.iter().enumerate() {
                utxos.insert((tx.txid(), vout), txout.clone());
            }
        });
        current_block.txdata.iter().skip(1).for_each(|tx| {
            let output_count = tx.output.len();
            tx.input.iter().enumerate().for_each(|(id, input)| {
                let assoc_output = utxos
                    .remove(&(
                        input.previous_output.txid,
                        input.previous_output.vout as usize,
                    ))
                    .unwrap();
                if id < output_count {
                    return;
                }
                let addr = match Address::from_script(
                    &assoc_output.script_pubkey,
                    bitcoincore_rpc::bitcoin::Network::Bitcoin,
                ) {
                    None => {
                        return;
                    }
                    Some(a) => a,
                };
                let sighash_byte = if assoc_output.script_pubkey.is_p2pkh()
                    || assoc_output.script_pubkey.is_p2pk()
                {
                    // p2pk example: d71fd2f64c0b34465b7518d240c00e83f6a5b10138a7079d1252858fe7e6b577
                    //p2pkh example: e03a9a4b5c557f6ee3400a29ff1475d1df73e9cddb48c2391abdc391d8c1504a
                    let sighash_byte =
                        match input.script_sig.instructions().next().unwrap().unwrap() {
                            Instruction::PushBytes(b) => b[b.len() - 1],

                            _ => {
                                return;
                            }
                        };
                    sighash_byte
                } else {
                    return;
                };

                let sighash = match SigHashType::from_u32_standard(sighash_byte as u32) {
                    Err(_) => {
                        info!(
                            "Found illegal scripthash! txid: {}, input_id: {}",
                            tx.txid(),
                            &id
                        );
                        return;
                    }
                    Ok(s) => s,
                };
                if sighash == SigHashType::Single || sighash == SigHashType::SinglePlusAnyoneCanPay
                {
                    info!("Found! txid: {}, input_id: {}", tx.txid(), id);
                    print_blockstream_txid_link(&tx.txid(), id);
                    print_blockstream_address_link(&addr);
                }
            });
        });
    });
}

fn print_blockstream_txid_link(txid: &Txid, input_id: usize) {
    info!(
        "https://blockstream.info/tx/{}?input:{}&expand",
        txid.to_string(),
        input_id
    );
}
fn print_blockstream_address_link(addr: &Address) {
    info!("https://blockstream.info/address/{}", addr.to_string())
}
