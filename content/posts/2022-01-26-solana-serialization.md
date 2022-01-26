---
title: "Playing with Solana Serialization"
date: 2022-01-26T12:03:43-05:00
draft: false
---

While exploring Solana, I wanted to get a better handle on how Solana serializes its own data. In order to do this [I built a tool](https://schwarzbi3r.github.io/solana-hack-n-hex) to quickly look at data on an account. Normally, consumers would have the RPC server parse the account data into its relevant fields. In my case, I'm looking at the raw data and ignoring any parsing in order to better understand the serialization handling.

## Let's get started with the hex viewer

To start, let's look at a program account. These break down into a 'program account' and a 'program data account' which actually holds the program's executable data.

In this case let's use Metaplex's token program[`metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`](https://schwarzbi3r.github.io/solana-hack-n-hex/#/mainnet-beta/account/metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s)

If we look at the raw data we can see it's

`02 00 00 00 05 E0 15 6C A7 D5 D4 E9 C8 42 21 BF 8A 4B D5 B6 BF 5E 54 4A 33 CD 53 CB 20 0B 23 49 3C B4 68 10`

This actually contains a serialized rust enum that looks like:

```rust
pub enum UpgradeableLoaderState {
    /// Account is not initialized.
    Uninitialized,
    /// A Buffer account.
    Buffer {
        /// Authority address
        authority_address: Option<Pubkey>,
        // The raw program data follows this serialized structure in the
        // account's data.
    },
    /// An Program account.
    Program {
        /// Address of the ProgramData account.
        programdata_address: Pubkey,
    },
    // A ProgramData account.
    ProgramData {
        /// Slot that the program was last modified.
        slot: u64,
        /// Address of the Program's upgrade authority.
        upgrade_authority_address: Option<Pubkey>,
        // The raw program data follows this serialized structure in the
        // account's data.
    },
}
```

Now it should become more clear how this got serialized. `02 00 00 00` is a `u32` denoting the 3rd (starting from 0x00) type in the enum, `Program`, which contains a Public key, which is `[u8; 32]` and starts from the 0x05.

Clicking the byte `0x05` at offset 0x04, we can see it gives us the key [`PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT`](https://schwarzbi3r.github.io/solana-hack-n-hex/#/mainnet-beta/account/PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT).

![bpf-loader-program](/2022-01-26-solana-serialization/bpf-loader-program-hex.png)

Then we pull up the program data at [`PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT`](https://schwarzbi3r.github.io/solana-hack-n-hex/#/mainnet-beta/account/PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT):


![bpf-loader-program-data](/2022-01-26-solana-serialization/bpf-loader-program-data-hex.png)

And while it's a whopping 694k of program data, we just want to pay attention to the header, which is defined in the enum as:

```rust
    // A ProgramData account.
    ProgramData {
        /// Slot that the program was last modified.
        slot: u64,
        /// Address of the Program's upgrade authority.
        upgrade_authority_address: Option<Pubkey>,
        // The raw program data follows this serialized structure in the
        // account's data.
    },
```


So as expected, we have `03 00 00 00` denoting the 4th enum (ProgramData), followed by a `u64` slot of `D1 12 F7 06 00 00 00 00` or 116855505 in base 10. Then we have a value of '01' which implies the 2nd enum in Option (Some), and we can verify that from the std::option code in rust, which is:

```rust
pub enum Option<T> {
    None,
    Some(T),
}
```

And finally, we have our 'upgrade_authority_address' which is a `[u8; 32]` of [`AqH29mZfQFgRpfwaPoTMWSKJ5kqauoc1FwVBRksZyQrt`](https://schwarzbi3r.github.io/solana-hack-n-hex/#/mainnet-beta/account/AqH29mZfQFgRpfwaPoTMWSKJ5kqauoc1FwVBRksZyQrt)

This all sounds reasonable, but can we replicate this serialization in Rust? It's actually pretty easy. It uses the libraries 'serde' and 'bincode' to do the actual serialization and deserialization, so with just a quick implementation of the enum along with a small test, we can reproduce the same data that we see showing up in Solana.

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct Pubkey([u8; 32]);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum UpgradeableLoaderState {
    /// Account is not initialized.
    Uninitialized,
    /// A Buffer account.
    Buffer {
        /// Authority address
        authority_address: Option<Pubkey>,
        // The raw program data follows this serialized structure in the
        // account's data.
    },
    /// An Program account.
    Program {
        /// Address of the ProgramData account.
        programdata_address: Pubkey,
    },
    // A ProgramData account.
    ProgramData {
        /// Slot that the program was last modified.
        slot: u64,
        /// Address of the Program's upgrade authority.
        upgrade_authority_address: Option<Pubkey>,
        // The raw program data follows this serialized structure in the
        // account's data.
    },
}

#[test]
fn test_serialization() {
    use bincode;
    use pretty_hex::*;

    // pubkey: 'AqH29mZfQFgRpfwaPoTMWSKJ5kqauoc1FwVBRksZyQrt'
    let pubkey: [u8; 32] = [
        0x92, 0x17, 0x02, 0xC4, 0x72, 0x5D, 0xC0, 0x41,
        0xF9, 0xDD, 0x8C, 0x51, 0x52, 0x60, 0x04, 0x26,
        0x00, 0x93, 0x0A, 0x0B, 0x02, 0x73, 0xDC, 0xFA,
        0x74, 0x92, 0x17, 0xFC, 0x94, 0xA2, 0x40, 0x49
        ];
    // The object that we will serialize.
    let target = UpgradeableLoaderState::ProgramData {
        slot: 116855505,
        upgrade_authority_address: Some(Pubkey(pubkey)),
    };

    let encoded: Vec<u8> = bincode::serialize(&target).unwrap();
    println!("{}", encoded.hex_dump());

    // The raw account data from AqH29mZfQFgRpfwaPoTMWSKJ5kqauoc1FwVBRksZyQrt
    // https://schwarzbi3r.github.io/solana-hack-n-hex/#/mainnet-beta/account/PwDiXFxQsGra4sFFTT8r1QWRMd4vfumiWC1jfWNfdYT
    let expected: [u8; 45] = [
        0x03, 0x00, 0x00, 0x00,
        0xD1, 0x12, 0xF7, 0x06, 0x00, 0x00, 0x00, 0x00,
        0x01,
        0x92, 0x17, 0x02, 0xC4, 0x72, 0x5D, 0xC0, 0x41,
        0xF9, 0xDD, 0x8C, 0x51, 0x52, 0x60, 0x04, 0x26,
        0x00, 0x93, 0x0A, 0x0B, 0x02, 0x73, 0xDC, 0xFA,
        0x74, 0x92, 0x17, 0xFC, 0x94, 0xA2, 0x40, 0x49
        ];
    assert_eq!(expected.to_vec(), encoded)
}
```

You can find the above code at [https://github.com/schwarzbi3r/schwarzbi3r.github.io/tree/main/code_examples/solana_serialization](https://github.com/schwarzbi3r/schwarzbi3r.github.io/tree/main/code_examples/solana_serialization)