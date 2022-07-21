#![allow(dead_code)]

type PlaceHolder = ();

// https://github.com/MinaProtocol/mina/blob/1765ba6bdfd7c454e5ae836c49979fa076de1bea/src/lib/mina_base/account.ml#L368
#[derive(Clone, Debug, Default)]
struct Account {
    pub public_key: PlaceHolder, // Public_key.Compressed.t
    pub token_id: PlaceHolder, // Token_id.t
    pub token_permissions: PlaceHolder, // Token_permissions.t
    pub token_symbol: PlaceHolder, // Token_symbol.t
    pub balance: PlaceHolder, // Balance.t
    pub nonce: PlaceHolder, // Nonce.t
    pub receipt_chain_hash: PlaceHolder, // Receipt.Chain_hash.t
    pub delegate: PlaceHolder, // Public_key.Compressed.t option
    pub voting_for: PlaceHolder, // State_hash.t
    pub timing: PlaceHolder, // Timing.t
    pub permissions: PlaceHolder, // Permissions.t
    pub zkapp: Option<PlaceHolder>, // Zkapp_account.t
    pub zkapp_uri: PlaceHolder, // string
}

#[derive(Clone, Debug)]
struct Hash {
    inner: Box<[u8; 32]>
}

#[derive(Clone, Debug)]
enum NodeOrLeaf {
    Leaf(Leaf),
    Node(Node),
}

#[derive(Clone, Debug, Default)]
struct Node {
    left: Option<Box<NodeOrLeaf>>,
    left_hash: Option<Hash>,
    right: Option<Box<NodeOrLeaf>>,
    right_hash: Option<Hash>,
}

#[derive(Clone, Debug, Default)]
struct Leaf {
    account: Account,
}

#[derive(Debug)]
struct Database {
    root: Option<NodeOrLeaf>,
    depth: u8
}

impl NodeOrLeaf {
    fn add_on_leafs(&mut self, new_elem_fun: &impl Fn() -> NodeOrLeaf) {
        let node = match self {
            NodeOrLeaf::Node(node) => node,
            NodeOrLeaf::Leaf(_leaf) => panic!("expected node"),
        };

        for child in &mut [&mut node.left, &mut node.right] {
            match child {
                Some(child) => {
                    child.add_on_leafs(new_elem_fun);
                },
                None => {
                    **child = Some(Box::new(new_elem_fun()));
                }
            }
        }
    }
}

impl Database {
    fn create(depth: u8) -> Self {
        assert!((1..0xfe).contains(&depth));

        let mut this = Self { root: Some(NodeOrLeaf::Node(Node::default())), depth };
        this.make_tree();

        this
    }

    fn make_tree(&mut self) {
        let root = self.root.as_mut().unwrap();

        // Add nodes
        for _ in 1..self.depth {
            root.add_on_leafs(&|| NodeOrLeaf::Node(Node::default()));
        }

        // Add leaves
        root.add_on_leafs(&|| NodeOrLeaf::Leaf(Leaf::default()));
    }

    fn naccounts(&self) -> usize {
        let mut naccounts = 0;
        self.naccounts_recursive(&self.root.as_ref().unwrap(), &mut naccounts);
        naccounts
    }

    fn naccounts_recursive(&self, elem: &NodeOrLeaf, naccounts: &mut usize) {
        match elem {
            NodeOrLeaf::Leaf(_) => *naccounts += 1,
            NodeOrLeaf::Node(node) => {
                self.naccounts_recursive(node.left.as_ref().unwrap(), naccounts);
                self.naccounts_recursive(node.right.as_ref().unwrap(), naccounts);
            },
        }
    }
}

fn main() {
    let db = Database::create(3);

    println!("Hello, world! {:#?}", db);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db() {
        let two: usize = 2;

        for depth in 2..25 {
            let db = Database::create(depth);
            let naccounts = db.naccounts();

            assert_eq!(naccounts, two.pow(depth as u32));
            eprintln!("depth={:?} naccounts={:?}", depth, naccounts);
        }
    }
}
