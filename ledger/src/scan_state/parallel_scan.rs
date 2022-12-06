use std::ops::ControlFlow;
use std::{fmt::Debug, rc::Rc};

use ControlFlow::{Break, Continue};

// type ControlFlow<T> = std::ops::ControlFlow<T, T>;

/// Sequence number for jobs in the scan state that corresponds to the order in
/// which they were added
#[derive(Debug)]
struct SequenceNumber(u64);

/// Each node on the tree is viewed as a job that needs to be completed. When a
/// job is completed, it creates a new "Todo" job and marks the old job as "Done"
#[derive(Debug)]
enum JobStatus {
    Todo,
    Done,
}

/// The number of new jobs- base and merge that can be added to this tree.
/// Each node has a weight associated to it and the
/// new jobs received are distributed across the tree based on this number.
#[derive(Debug)]
struct Weight {
    base: u64,
    merge: u64,
}

#[derive(Debug)]
struct BaseJob(); // TODO
#[derive(Debug)]
struct MergeJob(); // TODO

/// For base proofs (Proving new transactions)
mod base {
    use super::*;

    #[derive(Debug)]
    struct Record {
        job: BaseJob,
        seq_no: SequenceNumber,
        state: JobStatus,
    }

    #[derive(Debug)]
    enum Job {
        Empty,
        Full(Record),
    }

    #[derive(Debug)]
    pub(super) struct Base {
        weight: Weight,
        job: Job,
    }
}

/// For merge proofs: Merging two base proofs or two merge proofs
mod merge {
    use super::*;

    #[derive(Debug)]
    struct Record {
        left: MergeJob,
        right: MergeJob,
        seq_no: SequenceNumber,
        state: JobStatus,
    }

    #[derive(Debug)]
    enum Job {
        Empty,
        Part(MergeJob), // left
        Full(Record),
    }

    #[derive(Debug)]
    pub(super) struct Merge {
        weight_left: Weight,
        weight_right: Weight,
        job: Job,
    }
}

/// All the jobs on a tree that can be done. Base.Full and Merge.Full
#[derive(Debug)]
enum AvailableJob {
    Base(BaseJob),
    Merge { left: MergeJob, right: MergeJob },
}

/// New jobs to be added (including new transactions or new merge jobs)
#[derive(Debug)]
enum Job {
    Base(BaseJob),
    Merge(MergeJob),
}

/// Space available and number of jobs required to enqueue data.
/// first = space on the current tree and number of jobs required
/// to be completed
/// second = If the current-tree space is less than <max_base_jobs>
/// then remaining number of slots on a new tree and the corresponding
/// job count.
#[derive(Debug)]
struct SpaceSpartition {
    first: (u64, u64),
    second: Option<(u64, u64)>,
}

trait WithVTable<T>: Debug {
    fn by_ref(&self) -> &T;
}

impl<T: Debug> WithVTable<T> for T {
    fn by_ref(&self) -> &Self {
        self
    }
}

/// A single tree with number of leaves = max_base_jobs = 2**transaction_capacity_log_2
#[derive(Debug)]
enum Tree<B, M> {
    Leaf(B),
    Node {
        depth: u64,
        value: M,
        sub_tree: Rc<dyn WithVTable<Tree<(B, B), (M, M)>>>,
    },
}

#[derive(Debug)]
struct ParallelScan {
    trees: Vec<Tree<base::Base, merge::Merge>>,
    /// last emitted proof and the corresponding transactions
    acc: Option<(MergeJob, Vec<BaseJob>)>,
    /// Sequence number for the jobs added every block
    curr_job_seq_no: u64,
    /// transaction_capacity_log_2
    max_base_jobs: u64,
    delay: u64,
}

impl<B, M> Tree<B, M>
where
    B: Debug + 'static,
    M: Debug + 'static,
{
    /// mapi where i is the level of the tree
    fn map_depth<FunMerge, FunBase>(&self, fun_merge: FunMerge, fun_base: FunBase) -> Self
    where
        FunMerge: Fn(u64, &M) -> M,
        FunBase: Fn(&B) -> B,
    {
        match self {
            Tree::Leaf(base) => Self::Leaf(fun_base(&base)),
            Tree::Node {
                depth,
                value,
                sub_tree,
            } => Self::Node {
                depth: *depth,
                value: fun_merge(*depth, &value),
                sub_tree: {
                    let sub_tree: &Tree<(B, B), (M, M)> = (&**sub_tree).by_ref();

                    let sub_tree = sub_tree.map_depth(
                        |i, (x, y)| (fun_merge(i, x), fun_merge(i, y)),
                        |(x, y)| (fun_base(x), fun_base(y)),
                    );

                    Rc::new(sub_tree)
                },
            },
        }
    }

    fn map<FunMerge, FunBase>(&self, fun_merge: FunMerge, fun_base: FunBase) -> Self
    where
        FunMerge: Fn(&M) -> M,
        FunBase: Fn(&B) -> B,
    {
        self.map_depth(|_, m| fun_merge(m), fun_base)
    }

    fn fold_depth_until_prime<Accum, Final, FunMerge, FunBase>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
        init: Accum,
    ) -> ControlFlow<Final, Accum>
    where
        FunMerge: Fn(u64, Accum, &M) -> ControlFlow<Final, Accum>,
        FunBase: Fn(Accum, &B) -> ControlFlow<Final, Accum>,
    {
        match self {
            Tree::Leaf(base) => fun_base(init, base),
            Tree::Node {
                depth,
                value,
                sub_tree,
            } => {
                let accum = fun_merge(*depth, init, value)?;

                let sub_tree: &Tree<(B, B), (M, M)> = (&**sub_tree).by_ref();

                sub_tree.fold_depth_until_prime(
                    |i, accum, (x, y)| {
                        let accum = fun_merge(i, accum, x)?;
                        fun_merge(i, accum, y)
                    },
                    |accum, (x, y)| {
                        let accum = fun_base(accum, x)?;
                        fun_base(accum, y)
                    },
                    accum,
                )
            }
        }
    }

    fn fold_depth_until<Accum, Final, FunFinish, FunMerge, FunBase>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
        fun_finish: FunFinish,
        init: Accum,
    ) -> Final
    where
        FunMerge: Fn(u64, Accum, &M) -> ControlFlow<Final, Accum>,
        FunBase: Fn(Accum, &B) -> ControlFlow<Final, Accum>,
        FunFinish: Fn(Accum) -> Final,
    {
        match self.fold_depth_until_prime(fun_merge, fun_base, init) {
            Continue(accum) => fun_finish(accum),
            Break(value) => value,
        }
    }

    fn fold_depth<Accum, FunMerge, FunBase>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
        init: Accum,
    ) -> Accum
    where
        FunMerge: Fn(u64, Accum, &M) -> Accum,
        FunBase: Fn(Accum, &B) -> Accum,
    {
        self.fold_depth_until(
            |i, accum, a| Continue(fun_merge(i, accum, a)),
            |accum, d| Continue(fun_base(accum, d)),
            |x| x,
            init,
        )
    }

    fn fold<Accum, FunMerge, FunBase>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
        init: Accum,
    ) -> Accum
    where
        FunMerge: Fn(Accum, &M) -> Accum,
        FunBase: Fn(Accum, &B) -> Accum,
    {
        self.fold_depth(|_, accum, a| fun_merge(accum, a), fun_base, init)
    }

    fn fold_until<Accum, Final, FunFinish, FunMerge, FunBase>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
        fun_finish: FunFinish,
        init: Accum,
    ) -> Final
    where
        FunMerge: Fn(Accum, &M) -> ControlFlow<Final, Accum>,
        FunBase: Fn(Accum, &B) -> ControlFlow<Final, Accum>,
        FunFinish: Fn(Accum) -> Final,
    {
        self.fold_depth_until(
            |_, accum, a| fun_merge(accum, a),
            fun_base,
            fun_finish,
            init,
        )
    }

    fn update_split<Data, FunJobs, FunWeight, FunMerge, FunBase, Weight>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
        weight_merge: FunWeight,
        jobs: Data,
        update_level: u64,
        jobs_split: FunJobs,
    ) -> Result<Self, ()>
    where
        FunMerge: Fn(Data, u64, &M) -> Result<M, ()>,
        FunBase: Fn(Data, &B) -> Result<B, ()>,
        FunWeight: Fn(&M) -> (Weight, Weight),
        FunJobs: Fn((Weight, Weight), Data) -> (Data, Data),
        Data: Clone,
    {
        match self {
            Tree::Leaf(d) => fun_base(jobs, d).map(Self::Leaf),
            Tree::Node {
                depth,
                value,
                sub_tree,
            } => {
                let depth = *depth;
                let (weight_left_subtree, weight_right_subtree) = weight_merge(value);

                let value = fun_merge(jobs.clone(), depth, value)?;

                let sub_tree = if update_level == depth {
                    Rc::clone(&sub_tree)
                } else {
                    let new_jobs_list =
                        jobs_split((weight_left_subtree, weight_right_subtree), jobs);

                    let sub_tree: &Tree<(B, B), (M, M)> = (&**sub_tree).by_ref();

                    let sub_tree = sub_tree.update_split(
                        |(b1, b2), i, (x, y)| {
                            let left = fun_merge(b1, i, x)?;
                            let right = fun_merge(b2, i, y)?;
                            Ok((left, right))
                        },
                        |(b1, b2), (x, y)| {
                            let left = fun_base(b1, x)?;
                            let right = fun_base(b2, y)?;
                            Ok((left, right))
                        },
                        |(a, b)| (weight_merge(a), weight_merge(b)),
                        new_jobs_list,
                        update_level,
                        |(x, y), (a, b)| (jobs_split(x, a), jobs_split(y, b)),
                    )?;

                    Rc::new(sub_tree)
                };

                Ok(Self::Node {
                    depth,
                    value,
                    sub_tree,
                })
            }
        }
    }

    fn update_accumulate<Data, FunMerge, FunBase>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
    ) -> (Self, Data)
    where
        FunMerge: Fn((Data, Data), &M) -> (M, Data),
        FunBase: Fn(&B) -> (B, Data),
        Data: Clone,
    {
        fn transpose<A, B, C, D>((x1, y1): (A, B), (x2, y2): (C, D)) -> ((A, C), (B, D)) {
            ((x1, x2), (y1, y2))
        }

        match self {
            Tree::Leaf(d) => {
                let (new_base, count_list) = fun_base(d);
                (Self::Leaf(new_base), count_list)
            }
            Tree::Node {
                depth,
                value,
                sub_tree,
            } => {
                let sub_tree: &Tree<(B, B), (M, M)> = (&**sub_tree).by_ref();

                let (sub_tree, counts) = sub_tree.update_accumulate(
                    |(b1, b2), (x, y)| transpose(fun_merge(b1, x), fun_merge(b2, y)),
                    |(x, y)| transpose(fun_base(x), fun_base(y)),
                );

                let (value, count_list) = fun_merge(counts, value);

                let depth = *depth;
                let sub_tree = Rc::new(sub_tree);

                let tree = Self::Node {
                    depth,
                    value,
                    sub_tree,
                };
                (tree, count_list)
            }
        }
    }
}
