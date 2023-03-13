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

        #[id(6)]
        #[pbtype(sint64)]
        sint64: i64,

        #[id(7)]
        boolean: bool,

        #[id(8)]
        #[pbtype(fixed64)]
        fixed64: u64,

        #[id(9)]
        #[pbtype(sfixed64)]
        sfixed64: i64,

        #[id(10)]
        #[pbtype(fixed32)]
        fixed32: u32,

        #[id(11)]
        #[pbtype(sfixed32)]
        sfixed32: i32,

        #[id(12)]
        double: f64,

        #[id(13)]
        float: f32,
        /*
        "enum" => Some(Self::Enum),
        "string" => Some(Self::String),
        "bytes" => Some(Self::Bytes),
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
            sint64: -2,
            boolean: true,
            fixed64: 154,
            sfixed64: -3,
            fixed32: 155,
            sfixed32: -4,
            double: 3.14159,
            float: -1234.0,
        };
        let mut v = Vec::new();
        s.serialize(&mut v).unwrap();
        assert_eq!(
            v,
            vec![
                0x08, 0x96, 0x01, // int32
                0x10, 0x97, 0x01, // int64
                0x18, 0x98, 0x01, // uint32
                0x20, 0x99, 0x01, // uint64
                0x28, 0x01, // sint32
                0x30, 0x03, // sint64
                0x38, 0x01, // bool
                0x41, 0x9a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // fixed64
                0x49, 0xfd, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // sfixed64
                0x55, 0x9b, 0x00, 0x00, 0x00, // fixed32
                0x5d, 0xfc, 0xff, 0xff, 0xff, // sfixed32
                0x61, 0x6e, 0x86, 0x1b, 0xf0, 0xf9, 0x21, 0x09, 0x40, // double
                0x6d, 0x00, 0x40, 0x9a, 0xc4, // float
            ]
        );
    }
}
