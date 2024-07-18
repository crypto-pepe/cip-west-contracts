#![no_std]
#![no_main]

use we_cdk::{
    wevm::v0::bindings::{call_arg_string, call_contract},
    *,
};

const SEP: String = "__";
const FUNC_SEP: String = "####";
const KEY_INIT: String = "INIT";
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
fn _constructor(multisig: String, pauser: String, chain_id: Integer, signer_public_key: String) {
    require!(!contains_key!(KEY_INIT), "_constructor: already inited");
    require!(
        validate_contract(base58!(multisig)),
        "_constructor: inv multisig"
    );
    require!(
        validate_address(base58!(pauser)),
        "_constructor: inv pauser"
    );
    require!(chain_id > 0, "_constructor: inv chain_id");
    require!(
        base58!(signer_public_key).len() == PUBLIC_KEY_LENGTH,
        "_constructor: inv signer_public_key"
    );

    set_storage!(boolean::KEY_INIT => true);
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

    require!(!get_storage!(boolean::KEY_PAUSED), "execute: paused");
    require!(
        validate_contract(contract_address_bytes),
        "execute: inv contract"
    );
    require!(
        execution_chain_id == get_storage!(integer::KEY_CHAIN_ID),
        "execute: inv execution_chain_id"
    );

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
        to_bytes!(tx_hash.len() as i64),
        tx_hash,
        contract_address_bytes,
        to_bytes!(function_name.len() as i64),
        function_name.as_bytes(),
        to_bytes!(function_args_size),
        function_args_bytes
    );

    let data_hash = keccak256!(data_bytes);
    let data_hash_str = to_base58_string!(data_hash);
    let public_key = base58!(get_storage!(string::KEY_SIGNER_PUBLIC_KEY));
    let signature_bytes = base58!(signature);

    require!(
        signature_bytes.len() == SIGNATURE_LENGTH,
        "execute: inv signature len"
    );
    require!(
        sig_verify!(data_hash, signature_bytes, public_key),
        "execute: inv signature"
    );

    let data_hash_key = join!(string::KEY_DATA_HASH, SEP, data_hash_str);
    require!(!contains_key!(data_hash_key), "execute: duplicate data");

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
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    let old_signer_public_key_bytes = base58!(get_storage!(string::KEY_SIGNER_PUBLIC_KEY));
    let new_signer_public_key_bytes = base58!(new_signer_public_key);
    require!(
        new_signer_public_key_bytes.len() == PUBLIC_KEY_LENGTH,
        "update_signer: inv new_signer_public_key"
    );

    let old_signature_bytes = base58!(old_signature);
    let new_signature_bytes = base58!(new_signature);
    require!(
        old_signature_bytes.len() == SIGNATURE_LENGTH,
        "update_signer: inv old_signature len"
    );
    require!(
        new_signature_bytes.len() == SIGNATURE_LENGTH,
        "update_signer: inv new_signature len"
    );

    let old_data = join!(binary::"<<<PUBLIC--KEY--MIGRATION--ALLOWED>>>".as_bytes(), old_signer_public_key_bytes, new_signer_public_key_bytes);
    let new_data = join!(binary::"<<<PUBLIC--KEY--MIGRATION--CONFIRMED>>>".as_bytes(), old_signer_public_key_bytes, new_signer_public_key_bytes);

    require!(
        sig_verify!(old_data, old_signature_bytes, old_signer_public_key_bytes),
        "update_signer: inv old_signature"
    );
    require!(
        sig_verify!(new_data, new_signature_bytes, new_signer_public_key_bytes),
        "update_signer: inv new_signature"
    );

    set_storage!(string::KEY_SIGNER_PUBLIC_KEY => new_signer_public_key);
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
