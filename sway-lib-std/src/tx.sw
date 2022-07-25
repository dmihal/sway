//! Transaction field getters.
//! This will be replaced by instructions: https://github.com/FuelLabs/fuel-specs/issues/287
library tx;

use ::address::Address;
use ::context::registers::instrs_start;
use ::contract_id::ContractId;
use ::intrinsics::is_reference_type;
use ::mem::read;
use ::option::Option;

////////////////////////////////////////
// GTF Immediates
////////////////////////////////////////
const GTF_TYPE = 0x001;
const GTF_SCRIPT_GAS_PRICE = 0x002;
const GTF_SCRIPT_GAS_LIMIT = 0x003;
const GTF_SCRIPT_MATURITY = 0x004;
const GTF_SCRIPT_SCRIPT_LENGTH = 0x005;
const GTF_SCRIPT_SCRIPT_DATA_LENGTH = 0x006;
const GTF_SCRIPT_INPUTS_COUNT = 0x007;
const GTF_SCRIPT_OUTPUTS_COUNT = 0x008;
const GTF_SCRIPT_WITNESSES_COUNT = 0x009;
const GTF_SCRIPT_RECEIPTS_ROOT = 0x00A;
const GTF_SCRIPT_SCRIPT = 0x00B;
const GTF_SCRIPT_SCRIPT_DATA = 0x00C;
const GTF_SCRIPT_INPUT_AT_INDEX = 0x00D;
const GTF_SCRIPT_OUTPUT_AT_INDEX = 0x00E;

// const GTF_SCRIPT_WITNESS_AT_INDEX = 0x00F;

// const GTF_CREATE_GAS_PRICE = 0x010;
// const GTF_CREATE_GAS_LIMIT = 0x011;
// const GTF_CREATE_MATURITY = 0x012;
// const GTF_CREATE_BYTECODE_LENGTH = 0x013;
// const GTF_CREATE_BYTECODE_WITNESS_INDEX = 0x014;
// const GTF_CREATE_STORAGE_SLOTS_COUNT = 0x015;
// const GTF_CREATE_INPUTS_COUNT = 0x016;
// const GTF_CREATE_OUTPUTS_COUNT = 0x017;
// const GTF_CREATE_WITNESSES_COUNT = 0x018;
// const GTF_CREATE_SALT = 0x019;
// const GTF_CREATE_STORAGE_SLOT_AT_INDEX = 0x01A;
// const GTF_CREATE_INPUT_AT_INDEX = 0x01B;
// const GTF_CREATE_OUTPUT_AT_INDEX = 0x01C;
// const GTF_CREATE_WITNESS_AT_INDEX = 0x01D;

const GTF_INPUT_TYPE = 0x101;
const GTF_INPUT_COIN_TX_ID = 0x102;
// const GTF_INPUT_COIN_OUTPUT_INDEX = 0x103;
const GTF_INPUT_COIN_OWNER = 0x104;
// const GTF_INPUT_COIN_AMOUNT = 0x105;
// const GTF_INPUT_COIN_ASSET_ID = 0x106;
// const GTF_INPUT_COIN_TX_POINTER = 0x107;
// const GTF_INPUT_COIN_WITNESS_INDEX = 0x108;
// const GTF_INPUT_COIN_MATURITY = 0x109;
// const GTF_INPUT_COIN_PREDICATE_LENGTH = 0x10A;
// const GTF_INPUT_COIN_PREDICATE_DATA_LENGTH = 0x10B;
// const GTF_INPUT_COIN_PREDICATE = 0x10C;
const GTF_INPUT_COIN_PREDICATE_DATA = 0x10D;

const GTF_INPUT_CONTRACT_TX_ID = 0x10E;
// const GTF_INPUT_CONTRACT_OUTPUT_INDEX = 0x10F;
// const GTF_INPUT_CONTRACT_BALANCE_ROOT = 0x110;
// const GTF_INPUT_CONTRACT_STATE_ROOT = 0x111;
// const GTF_INPUT_CONTRACT_TX_POINTER = 0x112;
// const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;
// const GTF_INPUT_MESSAGE_MESSAGE_ID = 0x114;
// const GTF_INPUT_MESSAGE_SENDER = 0x115;
// const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;
// const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
// const GTF_INPUT_MESSAGE_NONCE = 0x118;

const GTF_INPUT_MESSAGE_OWNER = 0x119;
// const GTF_INPUT_MESSAGE_WITNESS_INDEX = 0x11A;
// const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11B;
// const GTF_INPUT_MESSAGE_PREDICATE_LENGTH = 0x11C;
// const GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH = 0x11D;
// const GTF_INPUT_MESSAGE_DATA = 0x11E;
// const GTF_INPUT_MESSAGE_PREDICATE = 0x11F;
const GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x120;

// const GTF_OUTPUT_COIN_TO = 0x202;
const GTF_OUTPUT_COIN_AMOUNT = 0x203;
// const GTF_OUTPUT_COIN_ASSET_ID = 0x204;
// const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x205;
// const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x206;
// const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x207;
// const GTF_OUTPUT_MESSAGE_RECIPIENT = 0x208;
const GTF_OUTPUT_MESSAGE_AMOUNT = 0x209;
// const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x20A;
// const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x20B;

// const GTF_WITNESS_DATA_LENGTH = 0x301;
// const GTF_WITNESS_DATA = 0x302;

// Input types
pub const INPUT_COIN = 0u8;
pub const INPUT_CONTRACT = 1u8;
pub const INPUT_MESSAGE = 2u8;

// Output types
pub const OUTPUT_COIN = 0u8;
pub const OUTPUT_CONTRACT = 1u8;
pub const OUTPUT_MESSAGE = 2u8;
pub const OUTPUT_CHANGE = 3u8;
pub const OUTPUT_VARIABLE = 4u8;
pub const OUTPUT_CONTRACT_CREATED = 5u8;

/// Get the transaction type.

// TODO: Use consts in place of immmediate values when https://github.com/FuelLabs/sway/issues/810 lands. can also refactor to have a single gtf() function and just pass the contants in as needed.
pub fn tx_type() -> u8 {
    // GTF_TYPE = 0x001
    asm(res) {
        gtf res zero i1;
        res: u8
    }
}

/// Get the transaction-script gas price.
pub fn tx_script_gas_price() -> u64 {
    // GTF_SCRIPT_GAS_PRICE = 0x002
    asm(res) {
        gtf res zero i2;
        res: u64
    }
}

/// Get the transaction-script gas limit.
pub fn tx_script_gas_limit() -> u64 {
    // GTF_SCRIPT_GAS_LIMIT = 0x003
    asm(res) {
        gtf res zero i3;
        res: u64
    }
}

/// Get the transaction maturity.
pub fn tx_maturity() -> u32 {
    // GTF_SCRIPT_MATURITY = 0x004;
    asm(res) {
        gtf res zero i4;
        res: u64
    }
}

/// Get the transaction script length.
pub fn tx_script_length() -> u64 {
    // GTF_SCRIPT_SCRIPT_LENGTH = 0x005
    asm(res) {
        gtf res zero i5;
        res: u64
    }
}

/// Get the transaction script data length.
pub fn tx_script_data_length() -> u64 {
    // GTF_SCRIPT_SCRIPT_DATA_LENGTH = 0x006
    asm(res) {
        gtf res zero i6;
        res: u64
    }
}

/// Get the transaction inputs count.
pub fn tx_inputs_count() -> u64 {
    // GTF_SCRIPT_INPUTS_COUNT = 0x007
    asm(res) {
        gtf res zero i7;
        res: u64
    }
}

/// Get the transaction outputs count.
pub fn tx_outputs_count() -> u64 {
    // GTF_SCRIPT_OUTPUTS_COUNT = 0x008
    asm(res) {
        gtf res zero i8;
        res: u64
    }
}

/// Get the transaction witnesses count.
pub fn tx_witnesses_count() -> u64 {
    // GTF_SCRIPT_WITNESSES_COUNT = 0x009
    asm(res) {
        gtf res zero i9;
        res: u64
    }
}

/// Get the transaction receipts root.
pub fn tx_receipts_root() -> b256 {
    // GTF_SCRIPT_RECEIPTS_ROOT = 0x00A
    read::<b256>(asm(res) {
        gtf res zero i10;
        res: u64
    }, 0)
}

////////////////////////////////////////
// Script
////////////////////////////////////////

/// Get the transaction script start pointer.
pub fn tx_script_start_pointer() -> u64 {
    // GTF_SCRIPT_SCRIPT = 0x00B
    asm(res) {
        gtf res zero i11;
        res: u64
    }
}

/// Get the transaction script data start pointer.
pub fn tx_script_data_start_pointer() -> u64 {
    // GTF_SCRIPT_SCRIPT_DATA = 0x00C
    asm(res) {
        gtf res zero i12;
        res: u64
    }
}

/// Get the script data, typed. Unsafe.
pub fn tx_script_data<T>() -> T {
    // TODO some safety checks on the input data? We are going to assume it is the right type for now.
    read::<T>(tx_script_data_start_pointer())
}

/// Get the script bytecode
/// Must be cast to a u64 array, with sufficient length to contain the bytecode.
/// Bytecode will be padded to next whole word.
pub fn tx_script_bytecode<T>() -> T {
    read::<T>(tx_script_start_pointer())
}

////////////////////////////////////////
// Inputs
////////////////////////////////////////

/// Get a pointer to an input given the index of the input.
pub fn tx_input_pointer(index: u64) -> u64 {
    // GTF_SCRIPT_INPUT_AT_INDEX = 0x00D
    asm(res, i: index) {
        gtf res i i13;
        res: u64
    }
}

/// Get the type of an input given a pointer to the input.
pub fn tx_input_type(index: u64) -> u8 {
    // GTF_INPUT_TYPE = 0x101
    asm(res, i: index) {
        gtf res i i257;
        res: u64
    }
}

/// If the input's type is `InputCoin` or `InputMessage`,
/// return the owner as an Option::Some(owner).
/// Otherwise, returns Option::None.
pub fn tx_input_owner(index: u64) -> Option<Address> {
    let type = tx_input_type(index);
    let owner_ptr = match type {
        // TODO: try using consts in match arms
        // 0 is the `Coin` Input type
        0u8 => {
            // GTF_INPUT_COIN_OWNER = 0x104
            asm(res, i: index) {
                gtf res i i260;
                res: u64
            }
        },
        // 2 is the `Message` Input type
        2u8 => {
            // GTF_INPUT_MESSAGE_OWNER = 0x119
            asm(res, i: index) {
                gtf res i i281;
                res: u64
            }
        },
        _ => {
            return Option::None;
        },
    };

    Option::Some(~Address::from(read::<b256>(
        owner_ptr,
        0
    )))
}

////////////////////////////////////////
// Inputs > Predicate
////////////////////////////////////////

pub fn predicate_data<T>(index: u64) -> T {
    let type = tx_input_type(index);
    let ptr = match type {
        // TODO: try using consts in match arms
        // 0 is the `Coin` Input type
        0u8 => {
            // GTF_INPUT_COIN_PREDICATE_DATA = 0x10D
            Option::Some(read::<T>(asm(res, i: index) {
                gtf res i i288;
                res: u64
            }), 0)
        },
        // 2 is the `Message` Input type
        2u8 => {
            // GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x120
            Option::Some(read::<T>(asm(res, i: index) {
                gtf res i i269;
                res: u64
            }), 0)
        },
        _ => {
            return Option::None;
        },
    };
}

////////////////////////////////////////
// Outputs
////////////////////////////////////////

/// Get a pointer to an output given the index of the output.
pub fn tx_output_pointer(index: u64) -> u64 {
    // GTF_SCRIPT_OUTPUT_AT_INDEX = 0x00E
    asm(res, i: index) {
        gtf res i i14;
        res: u64
    }
}

/// Get the type of an output at a given index
pub fn tx_output_type(index: u64) -> u8 {
    // GTF_OUTPUT_TYPE = 0x201
    asm(res, i: index) {
        gtf res i i513;
        res: u8
    }
}

/// Get the amount of coins to send for an output given a pointer to the output.
/// This method is only meaningful if the output type has the `amount` field.
/// Specifically: OutputCoin, OutputMessage, OutputChange, OutputVariable.
pub fn tx_output_amount(index: u64) -> Option<u64> {
    let type = tx_output_type(index);
    match type {
        // TODO: try using consts in match arms
        // 0 is the `Coin` Output type
        0u8 => {
            // GTF_OUTPUT_COIN_AMOUNT = 0x203
            Option::Some(asm(res, i: index) {
                gtf res i i515;
                res: u64
            })
        },
        // 2 is the `Message` Output type
        2u8 => {
            // GTF_OUTPUT_MESSAGE_AMOUNT = 0x209
            Option::Some(asm(res, i: index) {
                gtf res i i521;
                res: u64
            })
        },
        // 3 is the `Change` Output type
        // reusing the immediate for Message output type as there's no immediate for OutputChange
        3u8 => {
            // GTF_OUTPUT_MESSAGE_AMOUNT = 0x209
            Option::Some(asm(res, i: index) {
                gtf res i i521;
                res: u64
            })
        },
        // 4 is the `Variable` Output type
        // reusing the immediate for Message output type as there's no immediate for OutputVariable
        4u8 => {
            // GTF_OUTPUT_MESSAGE_AMOUNT = 0x209
            Option::Some(asm(res, i: index) {
                gtf res i i521;
                res: u64
            })
        },
        _ => {
            Option::None
        },
    }
}

/// Get the id of the current transaction.
pub fn tx_id(index: u64) -> Option<b256> {
    let type = tx_output_type(index);
    match type {
        // 0 is the `Coin` Input type
        0u8 => {
            // GTF_INPUT_COIN_TX_ID = 0x102
            Option::Some(read::<b256>(asm(res, i: index) {
                gtf res i i258;
                res: u64
            }))
        },
        // 1 is the `Contract` Input type
        1u8 => {
            // GTF_INPUT_CONTRACT_TX_ID = 0x10E
            Option::Some(read::<b256>(asm(res, i: index) {
                gtf res i i270;
                res: u64
            }))
        },
        _ => {
           Option::None
        },
    }
}
