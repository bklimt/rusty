pub use zombie_core::deserialize::read_tag;
pub use zombie_core::deserialize::read_uvarint;
pub use zombie_core::deserialize::Deserialize;
pub use zombie_core::deserialize::DeserializeError;
pub use zombie_core::deserialize::DeserializeField;
pub use zombie_core::proto_type::ProtoType;
pub use zombie_core::proto_type::WireType;
pub use zombie_core::serialize::write_tag;
pub use zombie_core::serialize::write_uvarint;
pub use zombie_core::serialize::Serialize;
pub use zombie_macro::Deserialize;
pub use zombie_macro::Serialize;

#[cfg(test)]
mod tests {
    mod zombie {
        pub use super::super::*;
    }
    use zombie::Deserialize;
    use zombie::DeserializeField;
    use zombie::Serialize;

    #[derive(Copy, Clone, Serialize, Deserialize)]
    enum TestEnum {
        VariantZero = 0,
        VariantOne = 1,
        VariantTwo = 2,
    }

    impl TryFrom<u64> for TestEnum {
        type Error = std::io::Error;

        fn try_from(value: u64) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(TestEnum::VariantZero),
                1 => Ok(TestEnum::VariantOne),
                2 => Ok(TestEnum::VariantTwo),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalud TestEnum value: {}", value),
                )),
            }
        }
    }

    #[derive(Serialize, Deserialize)]
    struct SubMessage {
        #[id(1)]
        int32: i32,
    }

    #[derive(Serialize, Deserialize)]
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

        #[id(14)]
        string: String,

        #[id(16)]
        bytes: Vec<u8>,

        #[id(18)]
        enumeration: TestEnum,

        #[id(19)]
        submessage: SubMessage,

        #[id(20)]
        repeated: Vec<u32>,
    }

    #[derive(Serialize)]
    struct TestOptionalMessage {
        #[id(1)]
        int32: Option<i32>,

        #[id(2)]
        int64: Option<i64>,

        #[id(3)]
        uint32: Option<u32>,

        #[id(4)]
        uint64: Option<u64>,

        #[id(5)]
        #[pbtype(sint32)]
        sint32: Option<i32>,

        #[id(6)]
        #[pbtype(sint64)]
        sint64: Option<i64>,

        #[id(7)]
        boolean: Option<bool>,

        #[id(8)]
        #[pbtype(fixed64)]
        fixed64: Option<u64>,

        #[id(9)]
        #[pbtype(sfixed64)]
        sfixed64: Option<i64>,

        #[id(10)]
        #[pbtype(fixed32)]
        fixed32: Option<u32>,

        #[id(11)]
        #[pbtype(sfixed32)]
        sfixed32: Option<i32>,

        #[id(12)]
        double: Option<f64>,

        #[id(13)]
        float: Option<f32>,

        #[id(14)]
        string: Option<String>,

        #[id(16)]
        bytes: Option<Vec<u8>>,

        #[id(18)]
        enumeration: Option<TestEnum>,

        #[id(19)]
        submessage: Option<SubMessage>,
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
            string: "hello".to_owned(),
            bytes: vec![1, 2, 3],
            enumeration: TestEnum::VariantTwo,
            submessage: SubMessage { int32: 150 },
            repeated: vec![156, 157, 158],
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
                0x72, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f, // string
                0x82, 0x01, 0x03, 0x01, 0x02, 0x03, // Vec<u8>
                0x90, 0x01, 0x02, // enum
                0x9a, 0x01, 0x03, 0x08, 0x96, 0x01, // submessage
                0xa0, 0x01, 0x9c, 0x01, 0xa0, 0x01, 0x9d, 0x01, 0xa0, 0x01, 0x9e,
                0x01, // repeated
            ]
        );
    }

    #[test]
    fn test_derive_optional_types() {
        let s = TestOptionalMessage {
            int32: Some(150),
            int64: Some(151),
            uint32: Some(152),
            uint64: Some(153),
            sint32: Some(-1),
            sint64: Some(-2),
            boolean: Some(true),
            fixed64: Some(154),
            sfixed64: Some(-3),
            fixed32: Some(155),
            sfixed32: Some(-4),
            double: Some(3.14159),
            float: Some(-1234.0),
            string: Some("hello".to_owned()),
            bytes: Some(vec![1, 2, 3]),
            enumeration: Some(TestEnum::VariantTwo),
            submessage: Some(SubMessage { int32: 150 }),
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
                0x72, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f, // string
                0x82, 0x01, 0x03, 0x01, 0x02, 0x03, // Vec<u8>
                0x90, 0x01, 0x02, // enum
                0x9a, 0x01, 0x03, 0x08, 0x96, 0x01, // submessage
            ]
        );
    }

    #[test]
    fn test_derive_optional_types_empty() {
        let s = TestOptionalMessage {
            int32: None,
            int64: None,
            uint32: None,
            uint64: None,
            sint32: None,
            sint64: None,
            boolean: None,
            fixed64: None,
            sfixed64: None,
            fixed32: None,
            sfixed32: None,
            double: None,

            float: None,
            string: None,
            bytes: None,
            enumeration: None,
            submessage: None,
        };
        let mut v = Vec::new();
        s.serialize(&mut v).unwrap();
        assert_eq!(v, vec![]);
    }

    #[test]
    fn test_derive_deserialize_types() {
        let v = vec![
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
            0x72, 0x05, 0x68, 0x65, 0x6c, 0x6c, 0x6f, // string
            0x82, 0x01, 0x03, 0x01, 0x02, 0x03, // Vec<u8>
            0x90, 0x01, 0x02, // enum
            0x9a, 0x01, 0x03, 0x08, 0x96, 0x01, // submessage
            0xa0, 0x01, 0x9c, 0x01, 0xa0, 0x01, 0x9d, 0x01, 0xa0, 0x01, 0x9e,
            0x01, // repeated
        ];

        // TODO(klimt): Zero this out.
        let mut actual = TestMessage {
            int32: 0,
            int64: 0,
            uint32: 0,
            uint64: 0,
            sint32: 0,
            sint64: 0,
            boolean: false,
            fixed64: 0,
            sfixed64: 0,
            fixed32: 0,
            sfixed32: 0,
            double: 0.0,
            float: 0.0,
            string: "".to_owned(),
            bytes: vec![],
            enumeration: TestEnum::VariantTwo,
            submessage: SubMessage { int32: 0 },
            repeated: vec![],
        };

        actual.deserialize(&mut &v[..]).unwrap();

        let expected = TestMessage {
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
            string: "hello".to_owned(),
            bytes: vec![1, 2, 3],
            enumeration: TestEnum::VariantTwo,
            submessage: SubMessage { int32: 150 },
            repeated: vec![156, 157, 158],
        };

        assert_eq!(expected.int32, actual.int32);
        assert_eq!(expected.int64, actual.int64);
        assert_eq!(expected.uint32, actual.uint32);
        assert_eq!(expected.uint64, actual.uint64);
        assert_eq!(expected.sint32, actual.sint32);
        assert_eq!(expected.sint64, actual.sint64);
        assert_eq!(expected.double, actual.double);
        assert_eq!(expected.float, actual.float);
        assert_eq!(expected.string, actual.string);
        assert_eq!(expected.bytes, actual.bytes);
        assert_eq!(expected.submessage.int32, actual.submessage.int32);
        assert_eq!(expected.repeated, actual.repeated);

        match actual.enumeration {
            TestEnum::VariantTwo => {}
            _ => panic!("incorrect variant"),
        }
    }
}
