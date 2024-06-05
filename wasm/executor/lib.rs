#![no_std]
#![no_main]

use we_cdk::{
    wevm::v0::bindings::{call_arg_string, call_contract},
    *,
};

const SEP: String = "__";
const FUNC_SEP: String = "####";
const KEY_THIS: String = "THIS";
const KEY_MULTISIG: String = "MULTISIG";
const KEY_STATUS: String = "STATUS";
const KEY_PAUSED: String = "PAUSED";
const KEY_PAUSER: String = "PAUSER";
const KEY_CHAIN_ID: String = "CHAIN_ID";
const KEY_SIGNER_PUBLIC_KEY: String = "SIGNER_PUBLIC_KEY";
const KEY_DATA_HASH: String = "DATA_HASH";

const FUNC_SEP_SIZE: i64 = 4;
const PUBLIC_KEY_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

fn validate_address(address: &[u8]) -> bool {
    // 86 for mainnet, 84 for testnet
    address.len() == 26 && (address.starts_with(&[1, 86]) || address.starts_with(&[1, 84]))
}

fn validate_contract(contract: &[u8]) -> bool {
    contract.len() == 32
}

#[action]
fn _constructor(multisig: String, pauser: String, chain_id: Integer, signer_public_key: String) {
    require!(validate_contract(base58!(multisig)));
    require!(validate_address(base58!(pauser)));
    require!(chain_id > 0);
    require!(base58!(signer_public_key).len() == PUBLIC_KEY_LENGTH);

    set_storage!(string::KEY_THIS => to_base58_string!(tx!(tx_id)));
    set_storage!(string::KEY_MULTISIG => multisig);
    set_storage!(boolean::KEY_PAUSED => false);
    set_storage!(string::KEY_PAUSER => pauser);
    set_storage!(integer::KEY_CHAIN_ID => chain_id);
    set_storage!(string::KEY_SIGNER_PUBLIC_KEY => signer_public_key);
}

#[action]
fn execute(
    contract: String,
    function_name: String,
    function_args: String,
    caller_chain_id: Integer,
    execution_chain_id: Integer,
    nonce: Integer,
    tx_hash: String,
    signature: String,
) {
    let contract_address_bytes = base58!(contract);
    let tx_hash_bytes = base58!(tx_hash);

    require!(!get_storage!(boolean::KEY_PAUSED));
    require!(validate_contract(contract_address_bytes));
    require!(execution_chain_id == get_storage!(integer::KEY_CHAIN_ID));

    // Serialize func args
    let mut function_args_mut = function_args.as_bytes();
    let mut function_args_bytes: &[u8] = &[0u8; 0];
    let mut function_args_size = 0;
    loop {
        let index = index_of!(function_args_mut, FUNC_SEP);
        if index == -1 {
            let arg = function_args_mut;
            call_arg_string(arg.as_ptr(), arg.len());
            function_args_bytes = join!(
                binary::function_args_bytes,
                to_bytes!(arg.len() as i64),
                arg
            );
            function_args_size += 1;
            break;
        }

        let arg = take!(function_args_mut, index);
        call_arg_string(arg.as_ptr(), arg.len());
        function_args_bytes = join!(
            binary::function_args_bytes,
            to_bytes!(arg.len() as i64),
            arg
        );
        function_args_mut = drop!(function_args_mut, index + FUNC_SEP_SIZE);
        function_args_size += 1;
    }

    // Serialize execution data
    let data_bytes = join!(
        binary::to_bytes!(caller_chain_id),
        to_bytes!(execution_chain_id),
        to_bytes!(nonce),
        to_bytes!(tx_hash_bytes.len() as i64),
        tx_hash_bytes,
        contract_address_bytes,
        to_bytes!(function_name.len() as i64),
        function_name.as_bytes(),
        to_bytes!(function_args_size),
        function_args_bytes
    );

    let data_hash = fast_hash!(data_bytes);
    let data_hash_str = to_base58_string!(data_hash);
    let public_key = base58!(get_storage!(string::KEY_SIGNER_PUBLIC_KEY));
    let signature_bytes = base58!(signature);

    require!(signature_bytes.len() == SIGNATURE_LENGTH);
    require!(sig_verify!(data_hash, signature_bytes, public_key));

    let data_hash_key = join!(string::KEY_DATA_HASH, SEP, data_hash_str);
    require!(!contains_key!(data_hash_key));

    call_contract(
        contract_address_bytes.as_ptr(),
        contract_address_bytes.len(),
        function_name.as_ptr(),
        function_name.len(),
    );

    set_storage!(integer::data_hash_key => block!(height));
}

#[action]
fn update_signer(new_signer_public_key: String, old_signature: String, new_signature: String) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    let old_signer_public_key_bytes = base58!(get_storage!(string::KEY_SIGNER_PUBLIC_KEY));
    let new_signer_public_key_bytes = base58!(new_signer_public_key);
    require!(new_signer_public_key_bytes.len() == PUBLIC_KEY_LENGTH);

    let old_signature_bytes = base58!(old_signature);
    let new_signature_bytes = base58!(new_signature);
    require!(old_signature_bytes.len() == SIGNATURE_LENGTH);
    require!(new_signature_bytes.len() == SIGNATURE_LENGTH);

    let old_data = join!(binary::"<<<PUBLIC--KEY--MIGRATION--ALLOWED>>>".as_bytes(), old_signer_public_key_bytes, new_signer_public_key_bytes);
    let new_data = join!(binary::"<<<PUBLIC--KEY--MIGRATION--CONFIRMED>>>".as_bytes(), old_signer_public_key_bytes, new_signer_public_key_bytes);

    require!(sig_verify!(
        old_data,
        old_signature_bytes,
        old_signer_public_key_bytes
    ));
    require!(sig_verify!(
        new_data,
        new_signature_bytes,
        new_signer_public_key_bytes
    ));

    set_storage!(string::KEY_SIGNER_PUBLIC_KEY => new_signer_public_key);
}

#[action]
fn pause() {
    let sender: String = to_base58_string!(tx!(sender));

    require!(equals!(string::sender, get_storage!(string::KEY_PAUSER)));
    require!(!get_storage!(boolean::KEY_PAUSED));

    set_storage!(boolean::KEY_PAUSED => true);
}

#[action]
fn unpause() {
    let sender: String = to_base58_string!(tx!(sender));

    require!(equals!(string::sender, get_storage!(string::KEY_PAUSER)));
    require!(get_storage!(boolean::KEY_PAUSED));

    set_storage!(boolean::KEY_PAUSED => false);
}

#[action]
fn update_pauser(new_pauser: String) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(validate_address(base58!(new_pauser)));

    set_storage!(string::KEY_PAUSER => new_pauser);
}

#[action]
fn update_multisig(new_multisig: String) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(validate_contract(base58!(new_multisig)));

    set_storage!(string::KEY_MULTISIG => new_multisig);
}
