#![no_std]
#![no_main]

use we_cdk::*;

const SEP: String = "__";
const FUNC_SEP: String = "####";
const KEY_THIS: String = "THIS";
const KEY_MULTISIG: String = "MULTISIG";
const KEY_STATUS: String = "STATUS";
const KEY_PAUSED: String = "PAUSED";
const KEY_PAUSER: String = "PAUSER";
const KEY_ALLOWANCE: String = "ALLOWANCE";
const KEY_CALL_CHAIN_ID: String = "CALL_CHAIN_ID";
const KEY_EVENT_SIZE: String = "EVENT_SIZE";
const KEY_EVENT: String = "EVENT";
const KEY_NONCE: String = "NONCE";

fn validate_address(address: &[u8]) -> bool {
    // 86 for mainnet, 84 for testnet
    address.len() == 26 && (address.starts_with(&[1, 86]) || address.starts_with(&[1, 84]))
}

fn validate_contract(contract: &[u8]) -> bool {
    contract.len() == 32
}

#[action]
fn _constructor(multisig: String, pauser: String, call_chain_id: Integer) {
    require!(validate_contract(base58!(multisig)));
    require!(validate_address(base58!(pauser)));
    require!(call_chain_id > 0);

    set_storage!(string::KEY_THIS => to_base58_string!(tx!(tx_id)));
    set_storage!(string::KEY_MULTISIG => multisig);
    set_storage!(boolean::KEY_PAUSED => false);
    set_storage!(string::KEY_PAUSER => pauser);
    set_storage!(integer::KEY_CALL_CHAIN_ID => call_chain_id);
    set_storage!(integer::KEY_NONCE => 0);
    set_storage!(integer::KEY_EVENT_SIZE => 0);
}

#[action]
fn call(
    execution_chain_id: Integer,
    execution_contract: String,
    function_name: String,
    function_args: String,
) {
    let caller: String = to_base58_string!(caller!());
    require!(caller.len() > 0);

    let allowance_key = join!(string::KEY_ALLOWANCE, SEP, caller);
    require!(contains_key!(allowance_key) && get_storage!(boolean::allowance_key));

    require!(!get_storage!(boolean::KEY_PAUSED));
    require!(execution_chain_id > 0);
    require!(!execution_contract.is_empty());
    require!(!function_name.is_empty());
    require!(!function_args.is_empty());

    let nonce: Integer = get_storage!(integer::KEY_NONCE);
    let event_size: Integer = get_storage!(integer::KEY_EVENT_SIZE);

    let call_chain_id: Integer = get_storage!(integer::KEY_CALL_CHAIN_ID);
    let block_height: Integer = block!(height);

    let function_args_with_caller = join!(string::caller, FUNC_SEP, function_args);
    let event = join!(
        string::to_string_int!(call_chain_id),
        SEP,
        to_string_int!(execution_chain_id),
        SEP,
        execution_contract,
        SEP,
        function_name,
        SEP,
        function_args_with_caller,
        SEP,
        to_string_int!(nonce),
        SEP,
        to_base58_string!(tx!(tx_id)),
        SEP,
        to_string_int!(block_height)
    );

    set_storage!(integer::KEY_EVENT_SIZE => event_size + 1);
    set_storage!(integer::KEY_NONCE => nonce + 1);
    set_storage!(string::join!(string::KEY_EVENT, SEP, to_string_int!(event_size)) => event);
}

#[action]
fn allow(caller: String) {
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);
    let tx_id = to_base58_string!(tx!(tx_id));

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(validate_contract(base58!(caller)));

    set_storage!(boolean::join!(string::KEY_ALLOWANCE, SEP, caller) => true);
}

#[action]
fn disallow(caller: String) {
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);
    let tx_id = to_base58_string!(tx!(tx_id));

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(validate_contract(base58!(caller)));

    set_storage!(boolean::join!(string::KEY_ALLOWANCE, SEP, caller) => false);
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
