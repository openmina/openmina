use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

use crate::versioned::Versioned;

///! Manually implemented Mina types.

pub type NonzeroCurvePointV1Versioned = Versioned<Versioned<NonzeroCurvePointV1, 1>, 1>;

pub type PublicKeyCompressedStableV1Versioned = NonzeroCurvePointV1Versioned;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, BinProtRead, BinProtWrite)]
pub struct NonzeroCurvePointV1 {
    x: crate::bigint::BigInt,
    is_odd: bool,
}

/// Location: [src/lib/parallel_scan/parallel_scan.ml:226:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L226)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0V1<MergeT, BaseT> {
    Leaf(Vec<BaseT>),
    Node {
        depth: i32,
        value: Vec<MergeT>,
        sub_tree:
            Box<TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0V1<MergeT, BaseT>>,
    },
}

#[derive(BinProtRead, BinProtWrite)]
enum _Tree {
    Leaf,
    Node,
}

impl<MergeT, BaseT> BinProtRead
    for TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0V1<MergeT, BaseT>
where
    MergeT: BinProtRead,
    BaseT: BinProtRead,
{
    fn binprot_read<R: std::io::Read + ?Sized>(r: &mut R) -> Result<Self, binprot::Error>
    where
        Self: Sized,
    {
        let mut depth: i32 = 0;
        let mut values: Vec<Vec<MergeT>> = Vec::new();
        loop {
            match _Tree::binprot_read(r)? {
                _Tree::Leaf => {
                    let len = 1 << depth;
                    let mut data = Vec::with_capacity(len);
                    for _ in 0..len {
                        data.push(BaseT::binprot_read(r)?)
                    }
                    let mut tree = Self::Leaf(data);
                    while let Some(value) = values.pop() {
                        depth = depth - 1;
                        tree = Self::Node {
                            depth,
                            value,
                            sub_tree: Box::new(tree),
                        }
                    }
                    return Ok(tree);
                }
                _Tree::Node => {
                    let _depth = i32::binprot_read(r)?;
                    if _depth != depth {
                        return Err(binprot::Error::CustomError(
                            format!("Incorrect tree depth, expected `{depth}`, got `{_depth}`")
                                .into(),
                        ));
                    }
                    let len = 1 << depth;
                    let mut value = Vec::with_capacity(len);
                    for _ in 0..len {
                        value.push(MergeT::binprot_read(r)?)
                    }
                    values.push(value);
                    depth += 1;
                }
            }
        }
    }
}

impl<MergeT, BaseT> BinProtWrite
    for TransactionSnarkScanStateStableV1VersionedV1PolyV1PolyV1TreesArg0V1<MergeT, BaseT>
where
    MergeT: BinProtWrite,
    BaseT: BinProtWrite,
{
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        let mut curr = self;
        let mut curr_depth = 0;
        loop {
            match curr {
                Self::Leaf(leaf) => {
                    let len = 1 << curr_depth;
                    if leaf.len() != len {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Invalid leaf data lenght, expecting {len}, got {}",
                                leaf.len()
                            ),
                        ));
                    }
                    _Tree::Leaf.binprot_write(w)?;
                    leaf.iter().try_for_each(|b| b.binprot_write(w))?;
                    return Ok(());
                }
                Self::Node {
                    depth,
                    value,
                    sub_tree,
                } => {
                    if depth != &curr_depth {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid depth, expecting {curr_depth}, got {depth}"),
                        ));
                    }
                    if value.len() != (1 << curr_depth) {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Invalid node data lenght, expecting {}, got {}",
                                1 << curr_depth,
                                value.len()
                            ),
                        ));
                    }
                    _Tree::Node.binprot_write(w)?;
                    depth.binprot_write(w)?;
                    value.iter().try_for_each(|b| b.binprot_write(w))?;
                    curr = sub_tree;
                    curr_depth += 1;
                }
            }
        }
    }
}
