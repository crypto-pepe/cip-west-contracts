#![no_std]
#![no_main]

use we_cdk::*;

const KEY_INIT: String = "INIT";
const KEY_OWNER: String = "OWNER";
const KEY_LAST_COMPLETED_MIGRATION: String = "LAST_COMPLETED_MIGRATION";

#[action]
fn _constructor() {
    require!(!contains_key!(KEY_INIT), "_constructor: already inited");

    set_storage!(boolean::KEY_INIT => true);
    set_storage!(string::KEY_OWNER => to_base58_string!(tx!(sender)));
    set_storage!(integer::KEY_LAST_COMPLETED_MIGRATION => 1);
}

#[action]
fn set_completed(completed: Integer) {
    let sender = to_base58_string!(tx!(sender));
    let owner = get_storage!(string::KEY_OWNER);
    require!(equals!(string::sender, owner), "set_completed: only owner");

    set_storage!(integer::KEY_LAST_COMPLETED_MIGRATION => completed);
}
