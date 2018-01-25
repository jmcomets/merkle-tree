pub trait Digest : PartialEq + Clone {
}

impl<T: PartialEq + Clone> Digest for T {
}

#[derive(Debug)]
pub enum HashTree<T, D: Digest> {
    Node((Vec<Box<HashTree<T, D>>>, D)),
    Leaf((T, D)),
}

pub fn leaf<D: Digest>(digest: D) -> HashTree<(), D> {
    HashTree::Leaf(((), digest))
}

pub fn node<D: Digest>(children: Vec<Box<HashTree<(), D>>>, digest: D) -> HashTree<(), D> {
    HashTree::Node((children, digest))
}

pub trait AggregateHasher<D> {
    fn aggregate_hash(&self, digests: &[&D]) -> D;

    // fn compute_digest(&self, data: &T) -> D;
}

impl<F, D> AggregateHasher<D> for F
    where F: Fn(&[&D]) -> D
{
    fn aggregate_hash(&self, digests: &[&D]) -> D {
        self.call((digests,))
    }
}

impl<T, D: Digest> HashTree<T, D> {
    pub fn is_consistent<H: AggregateHasher<D>>(&self, hasher: &H) -> bool {
        if let &HashTree::Node((ref children, ref stored_digest)) = self {
            debug_assert!(!children.is_empty());

            if children.iter().any(|c| !c.is_consistent(hasher)) {
                return false;
            }

            let digests: Vec<_> = children.iter()
                .map(|c| c.stored_digest())
                .collect();

            if !hasher.aggregate_hash(&digests).eq(stored_digest) {
                return false;
            }
        }

        true
    }

    fn stored_digest(&self) -> &D {
        match self {
            &HashTree::Node((_, ref digest)) => digest,
            &HashTree::Leaf((_, ref digest)) => digest,
        }
    }
}
