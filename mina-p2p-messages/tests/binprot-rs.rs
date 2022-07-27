use binprot::BinProtRead;
use binprot_derive::BinProtRead;
use mina_p2p_messages::versioned::Versioned;

type Foo = Versioned<FooV1, 1>;

#[derive(Debug, BinProtRead, PartialEq)]
struct FooV1(Vec<Bar>);

type Bar = Versioned<BarV1, 1>;

#[derive(Debug, BinProtRead, PartialEq)]
struct BarV1(u8);

#[test]
fn vec() {
    let bytes = b"\x01\x01\x01\x7f";
    let mut r = &bytes[..];
    let foo: Foo = BinProtRead::binprot_read(&mut r).unwrap();
    assert_eq!(foo, Versioned::from(FooV1(vec![Versioned::from(BarV1(0x7f))])));
}
