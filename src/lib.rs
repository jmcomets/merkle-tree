#![feature(slice_concat_ext)]
//#![feature(advanced_slice_patterns, slice_patterns)]

extern crate itertools;
extern crate generic_array;
extern crate tiger;

use std::fmt;
use std::slice::SliceConcatExt;

use itertools::Itertools;

use tiger::{Digest, Tiger};
use generic_array::GenericArray;
use generic_array::typenum::U24;

pub struct MerkleTree(Tree<Block, BlockHash>);

impl MerkleTree {
    pub fn new(digest: [u8; 24]) -> Self {
        MerkleTree(Tree::Node {
            left:   None,
            right:  None,
            digest: BlockHash(digest.into()),
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        assert!(!bytes.is_empty());

        let mut nodes: Vec<_> = bytes
            .chunks(BLOCK_SIZE)
            .map(|block_bytes| {
                let block = {
                    // could use some unsafety to speed things up
                    let mut block: Block = [0; BLOCK_SIZE];
                    block[..block_bytes.len()].copy_from_slice(block_bytes);
                    block
                };

                Tree::Leaf {
                    data:   Some(block),
                    digest: BlockHash::hashed(&block),
                }
            })
            .collect();

        while nodes.len() > 1 {
            nodes = nodes.into_iter()
                .chunks(2).into_iter()
                .map(|mut it| {
                    let a = it.next();
                    let b = it.next();

                    let digest = {
                        let a_digest = a.as_ref().map(|t| t.digest().0.as_slice()).unwrap();
                        let b_digest = b.as_ref().map(|t| t.digest().0.as_slice()).unwrap_or(&[]);

                        BlockHash::hashed(&[a_digest, b_digest].concat())
                    };

                    Tree::Node {
                        left:   a.map(Box::new),
                        right:  b.map(Box::new),
                        digest: digest,
                    }
                })
            .collect();
        }

        let root = nodes.pop().unwrap();
        MerkleTree(root)
    }

    pub fn is_consistent(&self) -> bool {
        self.0.is_consistent()
    }
}

impl Tree<Block, BlockHash> {
    fn is_consistent(&self) -> bool {
        match self {
            &Tree::Node { left: Some(ref left), ref right, ref digest } => {
                let computed_digest = {
                    let left_digest = left.digest().0.as_slice();
                    let right_digest = right.as_ref().map(|r| r.digest().0.as_slice()).unwrap_or(&[]);
                    let bytes = &[left_digest, right_digest].concat();
                    if bytes.is_empty() {
                        return false;
                    }

                    BlockHash::hashed(bytes)
                };

                *digest == computed_digest
                    && left.is_consistent()
                    && right.as_ref().map(|r| r.is_consistent()).unwrap_or(true)
            }

            &Tree::Leaf { ref digest, data: Some(ref data) } => {
                let computed_digest = BlockHash::hashed(data);
                *digest == computed_digest
            }

            _ => false
        }
    }
}

const BLOCK_SIZE: usize = 1024;
type Block = [u8; BLOCK_SIZE];

#[derive(PartialEq)]
struct BlockHash(GenericArray<u8, U24>);

impl BlockHash {
    fn hashed(bytes: &[u8]) -> Self {
        assert!(!bytes.is_empty());

        let mut hasher = Tiger::new();
        hasher.input(bytes);
        BlockHash(hasher.result())
    }
}

impl fmt::Debug for BlockHash {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_list()
            .entries(self.0.iter().cloned().map(HexByte))
            .finish()
    }
}

struct HexByte(u8);

impl fmt::Debug for HexByte {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:x}", self.0)
    }
}

enum Tree<T, D: PartialEq> {
    Node {
        left: Option<Box<Tree<T, D>>>,
        right: Option<Box<Tree<T, D>>>,
        digest: D,
    },
    Leaf {
        data: Option<T>,
        digest: D,
    },
}

impl<T, D: PartialEq + fmt::Debug> fmt::Debug for Tree<T, D> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Tree::Node { ref left, ref right, ref digest } => {
                fmt.debug_struct("Tree::Node")
                    .field("left", left)
                    .field("right", right)
                    .field("digest", digest)
                    .finish()
            }
            &Tree::Leaf { ref digest, .. } =>  {
                fmt.debug_struct("Tree::Leaf")
                    .field("digest", digest)
                    .finish()
            }
        }
    }
}

//enum TreePath {
//    Left,
//    Right,
//}

impl<T, D: PartialEq> Tree<T, D> {
    fn digest(&self) -> &D {
        match self {
            &Tree::Node { ref digest, .. } => digest,
            &Tree::Leaf { ref digest, .. } => digest,
        }
    }
}

//impl<T, D: PartialEq> Tree<T, D> {
//    fn get_mut(&mut self, path: &[TreePath]) -> Option<&mut Self> {
//        use TreePath::*;
//
//        match path {
//            &[]                  => Some(self),
//            &[Left,  ref rest..] => self.node_left_mut()
//                                        .and_then(|n| n.get_mut(rest)),
//            &[Right, ref rest..] => self.node_right_mut()
//                                        .and_then(|n| n.get_mut(rest)),
//        }
//    }
//
//    fn node_left_mut(&mut self) -> Option<&mut Self> {
//        if let &mut Tree::Node { ref mut left, .. } = self {
//            left.as_mut().map(|x| &mut**x)
//        } else {
//            None
//        }
//    }
//
//    fn node_right_mut(&mut self) -> Option<&mut Self> {
//        if let &mut Tree::Node { ref mut right, .. } = self {
//            right.as_mut().map(|x| &mut**x)
//        } else {
//            None
//        }
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_be_build_from_scratch() {
        let tree = MerkleTree::from_bytes(b"tototiti");
        assert!(tree.is_consistent());
    }

    #[test]
    fn it_can_be_expanded_in_steps() {
        let tree = MerkleTree::new([0; 24]);
    }
}
