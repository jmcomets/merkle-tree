#![feature(fn_traits)]

extern crate sha1;

use sha1::{Sha1, DIGEST_LENGTH};

pub mod hash_tree;

const BLOCK_SIZE: usize = 1024;

type Block = [u8; BLOCK_SIZE];
type HashDigest = [u8; DIGEST_LENGTH];

pub struct MerkleTree {
    data: Vec<Block>,
    tree: hash_tree::HashTree<usize, HashDigest>,
}

impl MerkleTree {
}

fn compute_hash_digest(bytes: &[u8]) -> HashDigest {
    let mut s = Sha1::new();
    s.update(bytes);
    s.digest().bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hash_tree::*;

    #[test]
    fn it_works() {
        let toto = b"toto";
        let titi = b"titi";

        let toto_digest = compute_hash_digest(toto);
        let titi_digest = compute_hash_digest(titi);

        let toto_node = leaf(toto_digest);
        let titi_node = leaf(titi_digest);

        let mut root_bytes = vec![];
        root_bytes.extend(toto_digest.into_iter());
        root_bytes.extend(titi_digest.into_iter());
        let root_digest = compute_hash_digest(root_bytes.as_slice());
        let root = node(vec![Box::new(toto_node), Box::new(titi_node)], root_digest);

        //println!("{:?}", root);
        assert!(root.is_consistent(&|digests: &[&HashDigest]| {
            let mut all_bytes = vec![];
            all_bytes.reserve(DIGEST_LENGTH * digests.len());

            for digest in digests {
                all_bytes.extend(digest.into_iter());
            }

            compute_hash_digest(all_bytes.as_slice())
        }));
    }
}
