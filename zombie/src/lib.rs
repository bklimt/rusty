pub use zombie_core::write_tag;
pub use zombie_core::write_uvarint;
pub use zombie_core::ProtoType;
pub use zombie_core::Serialize;
pub use zombie_core::WireType;
pub use zombie_macro::Serialize;

#[cfg(test)]
mod tests {
    use super::*;
    use zombie_core as zombie;

    #[derive(Serialize)]
    struct TestMessage {
        #[id(1)]
        int32: i32,

        #[id(2)]
        int64: i64,

        #[id(3)]
        uint32: u32,

        #[id(4)]
        uint64: u64,

        #[id(5)]
        #[pbtype(sint32)]
        sint32: i32,
        /*
        "sint32" => Some(Self::SInt32),
        "sint64" => Some(Self::SInt64),
        "bool" => Some(Self::Bool),
        "enum" => Some(Self::Enum),
        "fixed64" => Some(Self::Fixed64),
        "sfixed64" => Some(Self::SFixed64),
        "double" => Some(Self::Double),
        "string" => Some(Self::String),
        "bytes" => Some(Self::Bytes),
        "fixed32" => Some(Self::Fixed32),
        "sfixed32" => Some(Self::SFixed32),
        "float" => Some(Self::Float),
        */
    }

    #[test]
    fn test_derive_types() {
        let s = TestMessage {
            int32: 150,
            int64: 151,
            uint32: 152,
            uint64: 153,
            sint32: -1,
        };
        let mut v = Vec::new();
        s.serialize(&mut v).unwrap();
        assert_eq!(
            v,
            vec![
                /* int32  */ 0x08, 0x96, 0x01, /* int64  */ 0x10, 0x97, 0x01,
                /* uint32 */ 0x18, 0x98, 0x01, /* uint64 */ 0x20, 0x99, 0x01,
                /* sint32 */ 0x28, 0x01,
            ]
        );
    }
}
