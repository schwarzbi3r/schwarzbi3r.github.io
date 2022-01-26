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
