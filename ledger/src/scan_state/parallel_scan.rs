use std::ops::ControlFlow;
use std::{fmt::Debug, rc::Rc};

use sha2::digest::generic_array::GenericArray;
use sha2::digest::typenum::U32;
use sha2::{Digest, Sha256};
use ControlFlow::{Break, Continue};

// type ControlFlow<T> = std::ops::ControlFlow<T, T>;

/// Sequence number for jobs in the scan state that corresponds to the order in
/// which they were added
#[derive(Clone, Debug)]
struct SequenceNumber(u64);

/// Each node on the tree is viewed as a job that needs to be completed. When a
/// job is completed, it creates a new "Todo" job and marks the old job as "Done"
#[derive(Clone, Debug)]
enum JobStatus {
    Todo,
    Done,
}

impl JobStatus {
    fn to_string(&self) -> &'static str {
        match self {
            JobStatus::Todo => "Todo",
            JobStatus::Done => "Done",
        }
    }
}

/// The number of new jobs- base and merge that can be added to this tree.
/// Each node has a weight associated to it and the
/// new jobs received are distributed across the tree based on this number.
#[derive(Clone, Debug)]
struct Weight {
    base: u64,
    merge: u64,
}

impl Weight {
    fn zero() -> Self {
        Self { base: 0, merge: 0 }
    }
}

trait Lens {
    type Value;
    type Target;
    fn get<'a>(&self, target: &'a Self::Target) -> &'a Self::Value;
    fn set(&self, target: &Self::Target, value: Self::Value) -> Self::Target;
}

enum WeightLens {
    Base,
    Merge,
}

impl Lens for WeightLens {
    type Value = u64;
    type Target = Weight;

    fn get<'a>(&self, target: &'a Self::Target) -> &'a Self::Value {
        match self {
            WeightLens::Base => &target.base,
            WeightLens::Merge => &target.merge,
        }
    }

    fn set(&self, target: &Self::Target, value: Self::Value) -> Self::Target {
        match self {
            WeightLens::Base => Self::Target {
                base: value,
                merge: target.merge,
            },
            WeightLens::Merge => Self::Target {
                base: target.base,
                merge: value,
            },
        }
    }
}

#[derive(Clone, Debug)]
struct BaseJob(); // TODO
#[derive(Clone, Debug)]
struct MergeJob(); // TODO

/// For base proofs (Proving new transactions)
mod base {
    use super::*;

    #[derive(Clone, Debug)]
    pub(super) struct Record {
        pub job: BaseJob,
        pub seq_no: SequenceNumber,
        pub state: JobStatus,
    }

    #[derive(Clone, Debug)]
    pub(super) enum Job {
        Empty,
        Full(Record),
    }

    #[derive(Clone, Debug)]
    pub(super) struct Base {
        pub weight: Weight,
        pub job: Job,
    }

    impl Record {
        pub fn map<F: Fn(&BaseJob) -> BaseJob>(&self, fun: F) -> Self {
            Self {
                job: fun(&self.job),
                seq_no: self.seq_no.clone(),
                state: self.state.clone(),
            }
        }
    }

    impl Job {
        pub fn map<F: Fn(&BaseJob) -> BaseJob>(&self, fun: F) -> Self {
            match self {
                Job::Empty => Self::Empty,
                Job::Full(r) => Job::Full(r.map(fun)),
            }
        }
    }

    impl Base {
        pub fn map<F: Fn(&BaseJob) -> BaseJob>(&self, fun: F) -> Self {
            Self {
                weight: self.weight.clone(),
                job: self.job.map(fun),
            }
        }
    }
}

/// For merge proofs: Merging two base proofs or two merge proofs
mod merge {
    use super::*;

    #[derive(Clone, Debug)]
    pub(super) struct Record {
        pub left: MergeJob,
        pub right: MergeJob,
        pub seq_no: SequenceNumber,
        pub state: JobStatus,
    }

    #[derive(Clone, Debug)]
    pub(super) enum Job {
        Empty,
        Part(MergeJob), // left
        Full(Record),
    }

    #[derive(Clone, Debug)]
    pub(super) struct Merge {
        pub weight: (Weight, Weight),
        // pub weight_left: Weight,
        // pub weight_right: Weight,
        pub job: Job,
    }

    impl Record {
        pub fn map<F: Fn(&MergeJob) -> MergeJob>(&self, fun: F) -> Self {
            Self {
                left: fun(&self.left),
                right: fun(&self.right),
                seq_no: self.seq_no.clone(),
                state: self.state.clone(),
            }
        }
    }

    impl Job {
        pub fn map<F: Fn(&MergeJob) -> MergeJob>(&self, fun: F) -> Self {
            match self {
                Job::Empty => Self::Empty,
                Job::Part(j) => Job::Part(fun(j)),
                Job::Full(r) => Job::Full(r.map(fun)),
            }
        }
    }

    impl Merge {
        pub fn map<F: Fn(&MergeJob) -> MergeJob>(&self, fun: F) -> Self {
            Self {
                weight: self.weight.clone(),
                job: self.job.map(fun),
            }
        }
    }
}

/// All the jobs on a tree that can be done. Base.Full and Merge.Full
#[derive(Debug)]
enum AvailableJob {
    Base(BaseJob),
    Merge { left: MergeJob, right: MergeJob },
}

/// New jobs to be added (including new transactions or new merge jobs)
#[derive(Clone, Debug)]
enum Job {
    Base(BaseJob),
    Merge(MergeJob),
}

/// New jobs to be added (including new transactions or new merge jobs)
#[derive(Clone, Debug)]
enum HashableJob<B, M> {
    Base(B),
    Merge(M),
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
    curr_job_seq_no: SequenceNumber,
    /// transaction_capacity_log_2
    max_base_jobs: u64,
    delay: u64,
}

enum ResetKind {
    Base,
    Merge,
    Both,
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

    /// foldi where i is the cur_level
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

    fn update_split<Data, FunJobs, FunWeight, FunMerge, FunBase, Weight, R>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
        weight_merge: FunWeight,
        jobs: Data,
        update_level: u64,
        jobs_split: FunJobs,
    ) -> Result<(Self, Option<R>), ()>
    where
        FunMerge: Fn(Data, u64, M) -> Result<(M, Option<R>), ()>,
        FunBase: Fn(Data, B) -> Result<B, ()>,
        FunWeight: Fn(&M) -> (Weight, Weight),
        FunJobs: Fn((Weight, Weight), Data) -> (Data, Data),
        Data: Clone,
        M: Clone,
        B: Clone,
    {
        match self {
            Tree::Leaf(d) => {
                let res = fun_base(jobs, d.clone()).map(Self::Leaf)?;
                Ok((res, None))
            }
            Tree::Node {
                depth,
                value,
                sub_tree,
            } => {
                let depth = *depth;
                let (weight_left_subtree, weight_right_subtree) = weight_merge(value);
                // update the jobs at the current level
                let (value, scan_result) = fun_merge(jobs.clone(), depth, value.clone())?;
                // get the updated subtree
                let sub_tree = if update_level == depth {
                    Rc::clone(&sub_tree)
                } else {
                    // split the jobs for the next level
                    let new_jobs_list =
                        jobs_split((weight_left_subtree, weight_right_subtree), jobs);

                    let sub_tree: &Tree<(B, B), (M, M)> = (&**sub_tree).by_ref();

                    let sub_tree = sub_tree.update_split(
                        |(b1, b2), i, (x, y)| {
                            let left = fun_merge(b1, i, x)?;
                            let right = fun_merge(b2, i, y)?;
                            Ok(((left.0, right.0), left.1.zip(right.1)))
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

                    Rc::new(sub_tree.0)
                };

                Ok((
                    Self::Node {
                        depth,
                        value,
                        sub_tree,
                    },
                    scan_result,
                ))
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

                // get the updated subtree
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

impl Tree<base::Base, merge::Merge> {
    fn update<WeightLensSet>(
        &self,
        completed_jobs: Vec<Job>,
        update_level: u64,
        sequence_no: SequenceNumber,
        lens: WeightLens,
    ) -> Result<(Self, Option<MergeJob>), ()>
    where
        WeightLens: Fn(&Weight) -> u64,
        WeightLensSet: Fn(&Weight, u64) -> Weight,
    {
        let add_merges = |jobs: Vec<Job>,
                          current_level: u64,
                          merge_job: merge::Merge|
         -> Result<(merge::Merge, Option<MergeJob>), ()> {
            use merge::{
                Job::{Empty, Full, Part},
                Record,
            };
            use Job::{Base, Merge};

            let weight = merge_job.weight;
            let m = merge_job.job;

            let (w1, w2) = (&weight.0, &weight.1);
            let (left, right) = (*lens.get(w1), *lens.get(w2));

            if current_level == update_level - 1 {
                // Create new jobs from the completed ones
                let (new_weight, new_m) = match (&jobs[..], m) {
                    ([], m) => (weight, m),
                    ([Merge(a), Merge(b)], Empty) => {
                        let w1 = lens.set(w1, left - 1);
                        let w2 = lens.set(w2, right - 1);

                        (
                            (w1, w2),
                            Full(Record {
                                left: a.clone(),
                                right: b.clone(),
                                seq_no: sequence_no.clone(),
                                state: JobStatus::Todo,
                            }),
                        )
                    }
                    ([Merge(a)], Empty) => {
                        let w1 = lens.set(w1, left - 1);
                        let w2 = lens.set(w2, right);

                        ((w1, w2), Part(a.clone()))
                    }
                    ([Merge(b)], Part(a)) => {
                        let w1 = lens.set(w1, left);
                        let w2 = lens.set(w2, right - 1);

                        (
                            (w1, w2),
                            Full(Record {
                                left: a.clone(),
                                right: b.clone(),
                                seq_no: sequence_no.clone(),
                                state: JobStatus::Todo,
                            }),
                        )
                    }
                    ([Base(_)], Empty) => {
                        // Depending on whether this is the first or second of the two base jobs

                        let weight = if left == 0 {
                            let w1 = lens.set(w1, left);
                            let w2 = lens.set(w2, right - 1);
                            (w1, w2)
                        } else {
                            let w1 = lens.set(w1, left - 1);
                            let w2 = lens.set(w2, right);
                            (w1, w2)
                        };

                        (weight, Empty)
                    }
                    ([Base(_), Base(_)], Empty) => {
                        let w1 = lens.set(w1, left - 1);
                        let w2 = lens.set(w2, right - 1);

                        ((w1, w2), Empty)
                    }
                    (xs, m) => {
                        panic!(
                            "Got {} jobs when updating level {} and when one of the merge \
                             nodes at level {} is {:?}",
                            xs.len(),
                            update_level,
                            current_level,
                            m
                        );
                    }
                };

                Ok((
                    merge::Merge {
                        weight: new_weight,
                        job: new_m,
                    },
                    None::<MergeJob>,
                ))
            } else if current_level == update_level {
                // Mark completed jobs as Done

                match (jobs.as_slice(), m) {
                    (
                        [Merge(a)],
                        Full(
                            mut x @ Record {
                                state: JobStatus::Todo,
                                ..
                            },
                        ),
                    ) => {
                        x.state = JobStatus::Done;
                        let new_job = Full(x);

                        let (scan_result, weight) = if current_level == 0 {
                            let w1 = lens.set(w1, 0);
                            let w2 = lens.set(w2, 0);

                            (Some(a.clone()), (w1, w2))
                        } else {
                            (None, weight)
                        };

                        Ok((
                            merge::Merge {
                                weight,
                                job: new_job,
                            },
                            scan_result,
                        ))
                    }
                    ([], m) => Ok((merge::Merge { weight, job: m }, None)),
                    // ([], m) => Ok(((weight, m), None)),
                    (xs, m) => {
                        panic!(
                            "Got {} jobs when updating level {} and when one of the merge \
                             nodes at level {} is {:?}",
                            xs.len(),
                            update_level,
                            current_level,
                            m
                        );
                    }
                }
            } else if current_level < update_level - 1 {
                // Update the job count for all the level above
                match jobs.as_slice() {
                    [] => Ok((merge::Merge { weight, job: m }, None)),
                    _ => {
                        let jobs_length = jobs.len() as u64;
                        let jobs_sent_left = jobs_length.min(left);
                        let jobs_sent_right = (jobs_length - jobs_sent_left).min(right);

                        let w1 = lens.set(w1, left - jobs_sent_left);
                        let w2 = lens.set(w2, right - jobs_sent_right);
                        let weight = (w1, w2);

                        Ok((merge::Merge { weight, job: m }, None))
                    }
                }
            } else {
                Ok((merge::Merge { weight, job: m }, None))
            }
        };

        let add_bases = |jobs: Vec<Job>, base: base::Base| {
            use base::Job::{Empty, Full};
            use Job::{Base, Merge};

            let w = base.weight;
            let d = base.job;

            let weight = lens.get(&w);
            match (jobs.as_slice(), d) {
                ([], d) => Ok(base::Base { weight: w, job: d }),
                ([Base(d)], Empty) => {
                    let w = lens.set(&w, weight - 1);

                    Ok(base::Base {
                        weight: w,
                        job: Full(base::Record {
                            job: d.clone(),
                            seq_no: sequence_no.clone(),
                            state: JobStatus::Done,
                        }),
                    })
                }
                ([Merge(_)], Full(mut b)) => {
                    b.state = JobStatus::Done;

                    Ok(base::Base {
                        weight: w,
                        job: Full(b),
                    })
                }
                (xs, d) => {
                    panic!(
                        "Got {} jobs when updating level {} and when one of the base nodes \
                         is {:?}",
                        xs.len(),
                        update_level,
                        d
                    );
                }
            }
        };

        self.update_split(
            add_merges,
            add_bases,
            |merge| merge.weight.clone(),
            completed_jobs,
            update_level,
            |(w1, w2), a| {
                let l = *lens.get(&w1) as usize;
                let r = *lens.get(&w2) as usize;
                let a = a.as_slice();

                let take = |v: &[Job], n| v.iter().take(n).cloned().collect::<Vec<Job>>();
                let take_at =
                    |v: &[Job], skip, n| v.iter().skip(skip).take(n).cloned().collect::<Vec<Job>>();

                (take(a, l), take_at(a, l, r))
            },
        )
    }

    fn reset_weights(&self, reset_kind: ResetKind) -> Self {
        let fun_base = |base: &base::Base| {
            let set_one = |lens: WeightLens, weight: &Weight| lens.set(weight, 1);
            let set_zero = |lens: WeightLens, weight: &Weight| lens.set(weight, 0);

            use base::{
                Job::{Empty, Full},
                Record,
            };
            use JobStatus::Todo;

            let update_merge_weight = |weight: &Weight| {
                // When updating the merge-weight of base nodes, only the nodes with
                // "Todo" status needs to be included
                match &base.job {
                    Full(Record { state: Todo, .. }) => set_one(WeightLens::Merge, weight),
                    _ => set_zero(WeightLens::Merge, weight),
                }
            };

            let update_base_weight = |weight: &Weight| {
                // When updating the base-weight of base nodes, only the Empty nodes
                // need to be included
                match &base.job {
                    Empty => set_one(WeightLens::Base, weight),
                    Full(_) => set_zero(WeightLens::Base, weight),
                }
            };

            let weight = &base.weight;
            let (new_weight, dummy_right_for_base_nodes) = match reset_kind {
                ResetKind::Merge => (
                    update_merge_weight(&weight),
                    set_zero(WeightLens::Merge, &weight),
                ),
                ResetKind::Base => (
                    update_base_weight(&weight),
                    set_zero(WeightLens::Base, &weight),
                ),
                ResetKind::Both => {
                    let w = update_base_weight(weight);
                    (update_merge_weight(&w), Weight::zero())
                }
            };

            let base = base::Base {
                weight: new_weight.clone(),
                job: base.job.clone(),
            };

            (base, (new_weight, dummy_right_for_base_nodes))
        };

        let fun_merge = |lst: ((Weight, Weight), (Weight, Weight)), merge: &merge::Merge| {
            let ((w1, w2), (w3, w4)) = &lst;

            let reset = |lens: WeightLens, w: &Weight, ww: &Weight| {
                // Weights of all other jobs is sum of weights of its children
                (
                    lens.set(w, lens.get(w1) + lens.get(w2)),
                    lens.set(ww, lens.get(w3) + lens.get(w4)),
                )
            };

            use merge::{Job::Full, Record};
            use JobStatus::Todo;

            let ww = match reset_kind {
                ResetKind::Merge => {
                    // When updating the merge-weight of merge nodes, only the nodes
                    // with "Todo" status needs to be included
                    let lens = WeightLens::Merge;
                    match (&merge.weight, &merge.job) {
                        ((w1, w2), Full(Record { state: Todo, .. })) => {
                            (lens.set(w1, 1), lens.set(w2, 0))
                        }
                        ((w1, w2), _) => reset(lens, w1, w2),
                    }
                }
                ResetKind::Base => {
                    // The base-weight of merge nodes is the sum of weights of its
                    // children
                    let w = &merge.weight;
                    reset(WeightLens::Base, &w.0, &w.1)
                }
                ResetKind::Both => {
                    let w = &merge.weight;
                    let w = reset(WeightLens::Base, &w.0, &w.1);
                    reset(WeightLens::Merge, &w.0, &w.1)
                }
            };

            let merge = merge::Merge {
                weight: ww.clone(),
                job: merge.job.clone(),
            };

            (merge, ww)
        };

        let (result, _) = self.update_accumulate(fun_merge, fun_base);
        result
    }

    fn jobs_on_level(&self, depth: u64, level: u64) -> Vec<AvailableJob> {
        use JobStatus::Todo;

        self.fold_depth(
            |i, mut acc, a| {
                use merge::{Job::Full, Record};

                match (i == level, &a.job) {
                    (
                        true,
                        Full(Record {
                            left,
                            right,
                            state: Todo,
                            ..
                        }),
                    ) => {
                        let job = AvailableJob::Merge {
                            left: left.clone(),
                            right: right.clone(),
                        };
                        acc.push(job);
                    }
                    _ => {}
                };
                acc
            },
            |mut acc, d| {
                use base::{Job::Full, Record};

                match (level == depth, &d.job) {
                    (
                        true,
                        Full(Record {
                            job, state: Todo, ..
                        }),
                    ) => {
                        let job = AvailableJob::Base(job.clone());
                        acc.push(job);
                    }
                    _ => {}
                }
                acc
            },
            Vec::with_capacity(256),
        )
    }

    fn to_hashable_jobs(&self) -> Vec<HashableJob<base::Base, merge::Merge>> {
        use JobStatus::Done;

        self.fold(
            |mut acc, a| {
                match &a.job {
                    merge::Job::Full(merge::Record { state: Done, .. }) => {}
                    _ => {
                        acc.push(HashableJob::Merge(a.clone()));
                    }
                }
                acc
            },
            |mut acc, d| {
                match &d.job {
                    base::Job::Full(base::Record { state: Done, .. }) => {}
                    _ => {
                        acc.push(HashableJob::Base(d.clone()));
                    }
                }
                acc
            },
            Vec::with_capacity(256),
        )
    }

    fn jobs_records(&self) -> Vec<HashableJob<base::Record, merge::Record>> {
        self.fold(
            |mut acc, a: &merge::Merge| {
                match &a.job {
                    merge::Job::Full(x) => {
                        acc.push(HashableJob::Merge(x.clone()));
                    }
                    _ => {}
                }
                acc
            },
            |mut acc, d: &base::Base| {
                match &d.job {
                    base::Job::Full(j) => {
                        acc.push(HashableJob::Base(j.clone()));
                    }
                    _ => {}
                }
                acc
            },
            Vec::with_capacity(256),
        )
    }

    fn base_jobs(&self) -> Vec<BaseJob> {
        self.fold_depth(
            |_, acc, _| acc,
            |mut acc, d| {
                if let base::Job::Full(base::Record { job, .. }) = &d.job {
                    acc.push(job.clone());
                };
                acc
            },
            Vec::with_capacity(256),
        )
    }

    /// calculates the number of base and merge jobs that is currently with the Todo status
    fn todo_job_count(&self) -> (u64, u64) {
        use JobStatus::Todo;

        self.fold_depth(
            |_, (b, m), j| match &j.job {
                merge::Job::Full(merge::Record { state: Todo, .. }) => (b, m + 1),
                _ => (b, m),
            },
            |(b, m), d| match &d.job {
                base::Job::Full(base::Record { state: Todo, .. }) => (b + 1, m),
                _ => (b, m),
            },
            (0, 0),
        )
    }

    fn leaves(&self) -> Vec<base::Base> {
        self.fold_depth(
            |_, acc, _| acc,
            |mut acc, d| {
                if let base::Job::Full(_) = &d.job {
                    acc.push(d.clone());
                };
                acc
            },
            Vec::with_capacity(256),
        )
    }

    fn required_job_count(&self) -> u64 {
        match self {
            Tree::Node { value, .. } => {
                let (w1, w2) = &value.weight;
                w1.merge + w2.merge
            }
            Tree::Leaf(base) => base.weight.merge,
        }
    }

    fn available_space(&self) -> u64 {
        match self {
            Tree::Node { value, .. } => {
                let (w1, w2) = &value.weight;
                w1.base + w2.base
            }
            Tree::Leaf(base) => base.weight.base,
        }
    }

    fn create_tree_for_level(
        level: i64,
        depth: u64,
        merge_job: merge::Job,
        base_job: base::Job,
    ) -> Self {
        fn go<B, M>(d: u64, fun_merge: impl Fn(u64) -> M, base: B, depth: u64) -> Tree<B, M>
        where
            B: Debug + Clone + 'static,
            M: Debug + 'static,
        {
            if d >= depth {
                Tree::Leaf(base)
            } else {
                let sub_tree = go(
                    d + 1,
                    |i| (fun_merge(i), fun_merge(i)),
                    (base.clone(), base),
                    depth,
                );
                Tree::Node {
                    depth: d,
                    value: fun_merge(d),
                    sub_tree: Rc::new(sub_tree),
                }
            }
        }

        let base_weight = if level == -1 {
            Weight::zero()
        } else {
            Weight { base: 1, merge: 0 }
        };

        go(
            0,
            |d: u64| {
                let weight = if level == -1 {
                    (Weight::zero(), Weight::zero())
                } else {
                    let x = 2u64.pow(level as u32) / 2u64.pow(d as u32 + 1);
                    (Weight { base: x, merge: 0 }, Weight { base: x, merge: 0 })
                };
                merge::Merge {
                    weight,
                    job: merge_job.clone(),
                }
            },
            base::Base {
                weight: base_weight,
                job: base_job,
            },
            depth,
        )
    }

    fn create_tree(depth: u64) -> Self {
        let level: i64 = depth.try_into().unwrap();
        Self::create_tree_for_level(level, depth, merge::Job::Empty, base::Job::Empty)
    }
}

impl ParallelScan {
    fn with_leaner_trees(&self) -> Self {
        use JobStatus::Done;

        let trees = self
            .trees
            .iter()
            .map(|tree| {
                tree.map(
                    |merge_node| match &merge_node.job {
                        merge::Job::Full(merge::Record { state: Done, .. }) => merge::Merge {
                            weight: merge_node.weight.clone(),
                            job: merge::Job::Empty,
                        },
                        _ => merge_node.clone(),
                    },
                    |b| b.clone(),
                )
            })
            .collect();

        Self {
            trees,
            acc: self.acc.clone(),
            curr_job_seq_no: self.curr_job_seq_no.clone(),
            max_base_jobs: self.max_base_jobs,
            delay: self.delay,
        }
    }

    fn empty(max_base_jobs: u64, delay: u64) -> Self {
        let depth = ceil_log2(max_base_jobs);
        let first_tree = Tree::create_tree(depth);

        let mut trees = Vec::with_capacity(32);
        trees.push(first_tree);

        Self {
            trees,
            acc: None,
            curr_job_seq_no: SequenceNumber(0),
            max_base_jobs,
            delay,
        }
    }

    fn map<F1, F2>(&self, f1: F1, f2: F2) -> Self
    where
        F1: Fn(&MergeJob) -> MergeJob,
        F2: Fn(&BaseJob) -> BaseJob,
    {
        let trees = self
            .trees
            .iter()
            .map(|tree| tree.map_depth(|_, m| m.map(&f1), |a| a.map(&f2)))
            .collect();

        let acc = self
            .acc
            .as_ref()
            .map(|(m, bs)| (f1(m), bs.iter().map(&f2).collect()));

        Self {
            trees,
            acc,
            curr_job_seq_no: self.curr_job_seq_no.clone(),
            max_base_jobs: self.max_base_jobs,
            delay: self.delay,
        }
    }

    fn hash<FunMerge, FunBase>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
    ) -> GenericArray<u8, U32>
    where
        FunMerge: Fn(&MergeJob) -> String,
        FunBase: Fn(&BaseJob) -> String,
    {
        let Self {
            trees,
            acc,
            curr_job_seq_no,
            max_base_jobs,
            delay,
        } = self.with_leaner_trees();

        fn tree_hash<F1, F2>(
            tree: &Tree<base::Base, merge::Merge>,
            sha: &mut Sha256,
            mut fun_merge: F1,
            mut fun_base: F2,
        ) where
            F1: FnMut(&mut Sha256, &merge::Merge),
            F2: FnMut(&mut Sha256, &base::Base),
        {
            for job in tree.to_hashable_jobs() {
                match &job {
                    HashableJob::Base(base) => fun_base(sha, base),
                    HashableJob::Merge(merge) => fun_merge(sha, merge),
                }
            }
        }

        let mut sha: Sha256 = Sha256::new();

        trees.iter().for_each(|tree| {
            let w_to_string = |w: &Weight| format!("{}{}", w.base, w.merge);
            let ww_to_string =
                |(w1, w2): &(Weight, Weight)| format!("{}{}", w_to_string(w1), w_to_string(w2));

            let fun_merge = |sha: &mut Sha256, m: &merge::Merge| {
                let w = &m.weight;

                match &m.job {
                    merge::Job::Empty => {
                        let s = format!("{}Empty", ww_to_string(w));
                        sha.update(s);
                    }
                    merge::Job::Full(merge::Record {
                        left,
                        right,
                        seq_no,
                        state,
                    }) => {
                        let s = format!("{}Full{}{}", ww_to_string(w), seq_no.0, state.to_string());
                        sha.update(s);
                        sha.update(fun_merge(left));
                        sha.update(fun_merge(right));
                    }
                    merge::Job::Part(j) => {
                        let s = format!("{}Part", ww_to_string(w));
                        sha.update(s);
                        sha.update(fun_merge(j));
                    }
                }
            };

            let fun_base = |sha: &mut Sha256, m: &base::Base| {
                let w = &m.weight;

                match &m.job {
                    base::Job::Empty => {
                        sha.update(format!("{}Empty", w_to_string(w)));
                    }
                    base::Job::Full(base::Record { job, seq_no, state }) => {
                        let s = format!("{}Full{}{}", w_to_string(w), seq_no.0, state.to_string());
                        sha.update(s);
                        sha.update(fun_base(job));
                    }
                }
            };

            tree_hash(tree, &mut sha, fun_merge, fun_base);
        });

        match &acc {
            Some((a, d_lst)) => {
                let mut s = String::with_capacity(256);

                s.push_str(&fun_merge(a));
                for j in d_lst {
                    s.push_str(&fun_base(j));
                }

                sha.update(s);
            }
            None => {
                sha.update("None");
            }
        };

        sha.update(format!("{}", curr_job_seq_no.0));
        sha.update(format!("{}", max_base_jobs));
        sha.update(format!("{}", delay));

        sha.finalize()
    }
}

fn ceil_log2(n: u64) -> u64 {
    // let ceil_log2 i =
    //   if i <= 0
    //   then raise_s (Sexp.message "[Int.ceil_log2] got invalid input" [ "", sexp_of_int i ]);
    //   if i = 1 then 0 else num_bits - clz (i - 1)

    assert!(n > 0);
    if n == 1 {
        0
    } else {
        u64::BITS as u64 - (n - 1).leading_zeros() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceil_log2() {
        for a in 1..50u64 {
            let v = (a as f32).log2().ceil() as u64;
            let w = ceil_log2(a);
            // println!("{} {} {}", a, v, w);
            assert_eq!(v, w);
        }
    }
}
