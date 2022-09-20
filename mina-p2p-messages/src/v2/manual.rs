use binprot::{BinProtRead, BinProtWrite};
use binprot_derive::{BinProtRead, BinProtWrite};
use serde::{Deserialize, Serialize};

//
//  Location: [src/lib/parallel_scan/parallel_scan.ml:247:6](https://github.com/name-placeholder/mina/blob/da4c511501876adff40f3e1281392fedd121d607/src/lib/parallel_scan/parallel_scan.ml#L247)
//
//  Gid: 947
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TransactionSnarkScanStateStableV2PolyTreesArg0<MergeT, BaseT> {
    Leaf(Vec<BaseT>),
    Node {
        depth: i32,
        value: Vec<MergeT>,
        sub_tree: Box<TransactionSnarkScanStateStableV2PolyTreesArg0<MergeT, BaseT>>,
    },
}

#[derive(BinProtRead, BinProtWrite)]
enum _Tree {
    Leaf,
    Node,
}

impl<MergeT, BaseT> BinProtRead for TransactionSnarkScanStateStableV2PolyTreesArg0<MergeT, BaseT>
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

impl<MergeT, BaseT> BinProtWrite for TransactionSnarkScanStateStableV2PolyTreesArg0<MergeT, BaseT>
where
    MergeT: BinProtWrite,
    BaseT: BinProtWrite,
{
    fn binprot_write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        match self {
            Self::Leaf(leaf) => leaf.iter().try_for_each(|b| b.binprot_write(w)),
            Self::Node {
                depth,
                value,
                sub_tree,
            } => {
                depth.binprot_write(w)?;
                value.iter().try_for_each(|b| b.binprot_write(w))?;
                sub_tree.binprot_write(w)
            }
        }
    }
}
