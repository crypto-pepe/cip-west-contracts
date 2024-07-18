#![no_std]
#![no_main]

use we_cdk::*;

const SEP: String = "__";
const FUNC_SEP: String = "####";
const KEY_INIT: String = "INIT";
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

#[no_mangle]
#[inline(always)]
fn verify_multisig_confirmation() -> i32 {
    unsafe {
        let tx_id = to_base58_string!(tx!(tx_id));
        let this = get_storage!(string::KEY_THIS);
        let multisig = get_storage!(string::KEY_MULTISIG);

        let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
        require!(
            contains_key!(base58!(multisig) => status_key)
                && get_storage!(boolean::base58!(multisig) => status_key),
            "verify_multisig_confirmation: revert"
        );
    }

    0
}

#[action]
fn _constructor(multisig: String, pauser: String, call_chain_id: Integer) {
    require!(!contains_key!(KEY_INIT), "_constructor: already inited");
    require!(
        validate_contract(base58!(multisig)),
        "_constructor: inv multisig"
    );
    require!(
        validate_address(base58!(pauser)),
        "_constructor: inv pauser"
    );
    require!(call_chain_id > 0, "_constructor: inv call_chain_id");

    set_storage!(boolean::KEY_INIT => true);
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
    require!(caller.len() > 0, "call: caller is not contract");

    let allowance_key = join!(string::KEY_ALLOWANCE, SEP, caller);
    require!(
        contains_key!(allowance_key) && get_storage!(boolean::allowance_key),
        "call: not allowed"
    );

    require!(!get_storage!(boolean::KEY_PAUSED), "call: paused");
    require!(execution_chain_id > 0, "call: inv execution_chain_id");
    require!(
        !execution_contract.is_empty(),
        "call: inv execution_contract"
    );
    require!(!function_name.is_empty(), "call: inv function_name");
    require!(!function_args.is_empty(), "call: inv function_args");

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
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(validate_contract(base58!(caller)), "allow: inv caller");

    set_storage!(boolean::join!(string::KEY_ALLOWANCE, SEP, caller) => true);
}

#[action]
fn disallow(caller: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(validate_contract(base58!(caller)), "disallow: inv caller");

    set_storage!(boolean::join!(string::KEY_ALLOWANCE, SEP, caller) => false);
}

#[action]
fn pause() {
    let sender: String = to_base58_string!(tx!(sender));
    require!(to_base58_string!(caller!()).len() == 0);

    require!(
        equals!(string::sender, get_storage!(string::KEY_PAUSER)),
        "pause: not pauser"
    );
    require!(!get_storage!(boolean::KEY_PAUSED), "pause: paused");

    set_storage!(boolean::KEY_PAUSED => true);
}

#[action]
fn unpause() {
    let sender: String = to_base58_string!(tx!(sender));
    require!(to_base58_string!(caller!()).len() == 0);

    require!(
        equals!(string::sender, get_storage!(string::KEY_PAUSER)),
        "unpause: not pauser"
    );
    require!(get_storage!(boolean::KEY_PAUSED), "unpause: not paused");

    set_storage!(boolean::KEY_PAUSED => false);
}

#[action]
fn update_pauser(new_pauser: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        validate_address(base58!(new_pauser)),
        "update_pauser: inv pauser"
    );

    set_storage!(string::KEY_PAUSER => new_pauser);
}

#[action]
fn update_multisig(new_multisig: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        validate_contract(base58!(new_multisig)),
        "update_multisig: inv new_multisig"
    );

    set_storage!(string::KEY_MULTISIG => new_multisig);
}
