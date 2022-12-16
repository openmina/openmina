use std::collections::{BTreeMap, VecDeque};
use std::fmt::Debug;
use std::io::Write;
use std::ops::ControlFlow;

use sha2::digest::generic_array::GenericArray;
use sha2::digest::typenum::U32;
use sha2::{Digest, Sha256};
use ControlFlow::{Break, Continue};

/// Sequence number for jobs in the scan state that corresponds to the order in
/// which they were added
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct SequenceNumber(u64);

impl SequenceNumber {
    fn zero() -> Self {
        Self(0)
    }

    fn incr(&self) -> Self {
        Self(self.0 + 1)
    }

    fn is_u64_max(&self) -> bool {
        self.0 == u64::MAX
    }
}

impl std::ops::Sub for &'_ SequenceNumber {
    type Output = SequenceNumber;

    fn sub(self, rhs: &'_ SequenceNumber) -> Self::Output {
        SequenceNumber(self.0 - rhs.0)
    }
}

/// Each node on the tree is viewed as a job that needs to be completed. When a
/// job is completed, it creates a new "Todo" job and marks the old job as "Done"
#[derive(Clone, Debug)]
pub(super) enum JobStatus {
    Todo,
    Done,
}

impl JobStatus {
    fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Todo => "Todo",
            JobStatus::Done => "Done",
        }
    }
}

/// The number of new jobs- base and merge that can be added to this tree.
/// Each node has a weight associated to it and the
/// new jobs received are distributed across the tree based on this number.
#[derive(Clone)]
pub(super) struct Weight {
    base: u64,
    merge: u64,
}

impl Debug for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ base: {} merge: {} }}", self.base, self.merge)
    }
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

#[derive(Debug)]
enum WorkForTree {
    Current,
    Next,
}

/// For base proofs (Proving new transactions)
mod base {
    use super::*;

    #[derive(Clone, Debug)]
    pub(super) struct Record<BaseJob> {
        pub job: BaseJob,
        pub seq_no: SequenceNumber,
        pub state: JobStatus,
    }

    #[derive(Clone, Debug)]
    pub(super) enum Job<BaseJob> {
        Empty,
        Full(Record<BaseJob>),
    }

    #[derive(Clone, Debug)]
    pub(super) struct Base<BaseJob> {
        pub weight: Weight,
        pub job: Job<BaseJob>,
    }

    impl<BaseJob: Clone> Record<BaseJob> {
        pub fn map<F: Fn(&BaseJob) -> BaseJob>(&self, fun: F) -> Self {
            Self {
                job: fun(&self.job),
                seq_no: self.seq_no.clone(),
                state: self.state.clone(),
            }
        }

        pub fn with_seq_no(&self, no: SequenceNumber) -> Self {
            Self {
                seq_no: no,
                state: self.state.clone(),
                job: self.job.clone(),
            }
        }
    }

    impl<BaseJob: Clone> Job<BaseJob> {
        pub fn map<F: Fn(&BaseJob) -> BaseJob>(&self, fun: F) -> Self {
            match self {
                Job::Empty => Self::Empty,
                Job::Full(r) => Job::Full(r.map(fun)),
            }
        }
    }

    impl<BaseJob: Clone> Base<BaseJob> {
        pub fn map<F: Fn(&BaseJob) -> BaseJob>(&self, fun: F) -> Self {
            Self {
                weight: self.weight.clone(),
                job: self.job.map(fun),
            }
        }

        pub fn with_seq_no(&self, no: SequenceNumber) -> Self {
            Self {
                weight: self.weight.clone(),
                job: match &self.job {
                    Job::Full(record) => Job::Full(record.with_seq_no(no)),
                    x => x.clone(),
                },
            }
        }
    }
}

/// For merge proofs: Merging two base proofs or two merge proofs
mod merge {
    use super::*;

    #[derive(Clone, Debug)]
    pub(super) struct Record<MergeJob> {
        pub left: MergeJob,
        pub right: MergeJob,
        pub seq_no: SequenceNumber,
        pub state: JobStatus,
    }

    impl<MergeJob: Clone> Record<MergeJob> {
        pub fn with_seq_no(&self, no: SequenceNumber) -> Self {
            Self {
                seq_no: no,
                left: self.left.clone(),
                right: self.right.clone(),
                state: self.state.clone(),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub(super) enum Job<MergeJob> {
        Empty,
        Part(MergeJob), // left
        Full(Record<MergeJob>),
    }

    #[derive(Clone, Debug)]
    pub(super) struct Merge<MergeJob> {
        pub weight: (Weight, Weight),
        pub job: Job<MergeJob>,
    }

    impl<MergeJob> Record<MergeJob> {
        pub fn map<F: Fn(&MergeJob) -> MergeJob>(&self, fun: F) -> Self {
            Self {
                left: fun(&self.left),
                right: fun(&self.right),
                seq_no: self.seq_no.clone(),
                state: self.state.clone(),
            }
        }
    }

    impl<MergeJob> Job<MergeJob> {
        pub fn map<F: Fn(&MergeJob) -> MergeJob>(&self, fun: F) -> Self {
            match self {
                Job::Empty => Self::Empty,
                Job::Part(j) => Job::Part(fun(j)),
                Job::Full(r) => Job::Full(r.map(fun)),
            }
        }
    }

    impl<MergeJob: Clone> Merge<MergeJob> {
        pub fn map<F: Fn(&MergeJob) -> MergeJob>(&self, fun: F) -> Self {
            Self {
                weight: self.weight.clone(),
                job: self.job.map(fun),
            }
        }

        pub fn with_seq_no(&self, no: SequenceNumber) -> Self {
            Self {
                weight: self.weight.clone(),
                job: match &self.job {
                    Job::Full(record) => Job::Full(record.with_seq_no(no)),
                    x => x.clone(),
                },
            }
        }
    }
}

/// All the jobs on a tree that can be done. Base.Full and Merge.Full
#[derive(Debug)]
pub enum AvailableJob<BaseJob, MergeJob> {
    Base(BaseJob),
    Merge { left: MergeJob, right: MergeJob },
}

/// New jobs to be added (including new transactions or new merge jobs)
#[derive(Clone, Debug)]
enum Job<BaseJob, MergeJob> {
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
pub struct SpacePartition {
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

#[derive(Clone, Debug)]
enum Value<B, M> {
    Leaf(B),
    Node(M),
}

/// A single tree with number of leaves = max_base_jobs = 2**transaction_capacity_log_2
#[derive(Clone)]
struct Tree<B, M> {
    values: Vec<Value<B, M>>,
}

impl<B, M> Debug for Tree<base::Base<B>, merge::Merge<M>>
where
    B: Debug,
    M: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        enum BaseOrMerge<'a, B, M> {
            Base(&'a base::Base<B>),
            Merge(&'a merge::Merge<M>),
        }

        impl<'a, B, M> Debug for BaseOrMerge<'a, B, M>
        where
            B: Debug,
            M: Debug,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Base(arg0) => write!(f, "{:?}", arg0),
                    Self::Merge(arg0) => write!(f, "{:?}", arg0),
                }
            }
        }

        let mut by_depth = BTreeMap::<usize, Vec<_>>::default();

        for (index, v) in self.values.iter().enumerate() {
            let vec = by_depth.entry(btree::depth_at(index) as usize).or_default();
            let v = match v {
                Value::Leaf(b) => BaseOrMerge::Base(b),
                Value::Node(m) => BaseOrMerge::Merge(m),
            };
            vec.push(v);
        }

        for (depth, values) in by_depth.iter() {
            writeln!(f, "depth={} {:#?}", depth, values)?;
        }

        Ok(())
    }
}

mod btree {
    // https://stackoverflow.com/a/31147495/5717561
    pub fn depth_at(index: usize) -> u64 {
        // Get the depth from its index (in the array)
        // TODO: Find if there is a faster way to get that
        let depth = ((index + 1) as f32).log2().floor() as u32;
        depth as u64
    }

    pub fn child_left(index: usize) -> usize {
        (index * 2) + 1
    }

    pub fn child_right(index: usize) -> usize {
        (index * 2) + 2
    }

    pub fn parent(index: usize) -> Option<usize> {
        Some(index.checked_sub(1)? / 2)
    }

    pub fn range_at_depth(depth: u64) -> std::ops::Range<usize> {
        if depth == 0 {
            return 0..1;
        }

        let start = (1 << depth) - 1;
        let end = (1 << (depth + 1)) - 1;

        start..end
    }
}

impl<B, M> Tree<B, M>
where
    B: Debug + 'static,
    M: Debug + 'static,
{
    /// mapi where i is the level of the tree
    fn map_depth<FunMerge, FunBase>(&self, fun_merge: &FunMerge, fun_base: &FunBase) -> Self
    where
        FunMerge: for<'a> Fn(u64, &'a M) -> M,
        FunBase: for<'a> Fn(&'a B) -> B,
    {
        let values = self
            .values
            .iter()
            .enumerate()
            .map(|(index, value)| match value {
                Value::Leaf(base) => Value::Leaf(fun_base(base)),
                Value::Node(merge) => Value::Node(fun_merge(btree::depth_at(index), merge)),
            })
            .collect();

        Self { values }
    }

    fn map<FunMerge, FunBase>(&self, fun_merge: FunMerge, fun_base: FunBase) -> Self
    where
        FunMerge: Fn(&M) -> M,
        FunBase: Fn(&B) -> B,
    {
        self.map_depth(&|_, m| fun_merge(m), &fun_base)
    }

    /// foldi where i is the cur_level
    fn fold_depth_until_prime<Accum, Final, FunMerge, FunBase>(
        &self,
        fun_merge: &FunMerge,
        fun_base: &FunBase,
        init: Accum,
    ) -> ControlFlow<Final, Accum>
    where
        FunMerge: Fn(u64, Accum, &M) -> ControlFlow<Final, Accum>,
        FunBase: Fn(Accum, &B) -> ControlFlow<Final, Accum>,
    {
        let mut accum = init;

        for (index, value) in self.values.iter().enumerate() {
            accum = match value {
                Value::Leaf(base) => fun_base(accum, base)?,
                Value::Node(merge) => fun_merge(btree::depth_at(index), accum, merge)?,
            };
        }

        Continue(accum)
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
        match self.fold_depth_until_prime(&fun_merge, &fun_base, init) {
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
        fun_merge: &FunMerge,
        fun_base: &FunBase,
        weight_merge: &FunWeight,
        jobs: &[Data],
        update_level: u64,
        jobs_split: &FunJobs,
    ) -> Result<(Self, Option<R>), ()>
    where
        FunMerge: Fn(&[Data], u64, M) -> Result<(M, Option<R>), ()>,
        FunBase: Fn(&[Data], B) -> Result<B, ()>,
        FunWeight: Fn(&M) -> (Weight, Weight),
        FunJobs: Fn((Weight, Weight), &[Data]) -> (&[Data], &[Data]),
        Data: Clone,
        M: Clone,
        B: Clone,
    {
        let mut values = Vec::with_capacity(self.values.len());
        let mut scan_result = None;

        // Because our tree is a perfect binary tree, two values pushed
        // at the back of `jobs_fifo` by a node will be popped by its
        // left and right children, respectively
        let mut jobs_fifo = VecDeque::with_capacity(self.values.len());

        jobs_fifo.push_back(jobs);

        for (index, value) in self.values.iter().enumerate() {
            let depth = btree::depth_at(index);

            if depth > update_level {
                values.push(value.clone());
                continue;
            }

            let jobs_for_this = jobs_fifo.pop_front().unwrap();

            let value = match value {
                Value::Leaf(base) => Value::Leaf(fun_base(jobs_for_this, base.clone())?),
                Value::Node(merge) => {
                    let (jobs_left, jobs_right) = jobs_split(weight_merge(merge), jobs_for_this);
                    jobs_fifo.push_back(jobs_left);
                    jobs_fifo.push_back(jobs_right);

                    let (value, result) = fun_merge(jobs_for_this, depth, merge.clone())?;

                    if scan_result.is_none() {
                        scan_result = Some(result);
                    }

                    Value::Node(value)
                }
            };

            values.push(value);
        }

        assert_eq!(jobs_fifo.capacity(), self.values.len());

        Ok(((Self { values }), scan_result.flatten()))
    }

    fn update_accumulate<Data, FunMerge, FunBase>(
        &self,
        fun_merge: &FunMerge,
        fun_base: &FunBase,
    ) -> (Self, Data)
    where
        FunMerge: Fn((Data, Data), &M) -> (M, Data),
        FunBase: Fn(&B) -> (B, Data),
        Data: Clone,
    {
        let mut datas = vec![None; self.values.len()];

        let childs_of = |data: &mut [Option<Data>], index: usize| -> Option<(Data, Data)> {
            let left = data
                .get_mut(btree::child_left(index))
                .and_then(Option::take)?;
            let right = data
                .get_mut(btree::child_right(index))
                .and_then(Option::take)?;

            Some((left, right))
        };

        let mut values: Vec<_> = self
            .values
            .iter()
            .enumerate()
            .rev()
            .map(|(index, value)| match value {
                Value::Leaf(base) => {
                    let (new_base, count_list) = fun_base(base);
                    datas[index] = Some(count_list);
                    Value::Leaf(new_base)
                }
                Value::Node(merge) => {
                    let (left, right) = childs_of(datas.as_mut(), index).unwrap();
                    let (value, count_list) = fun_merge((left, right), merge);
                    datas[index] = Some(count_list);
                    Value::Node(value)
                }
            })
            .collect();

        values.reverse();

        (Self { values }, datas[0].take().unwrap())
    }

    fn iter(&self) -> impl Iterator<Item = (u64, &Value<B, M>)> {
        self.values
            .iter()
            .enumerate()
            .map(|(index, value)| (btree::depth_at(index), value))
    }
}

#[derive(Clone, Debug)]
pub(super) struct ParallelScan<BaseJob, MergeJob> {
    trees: Vec<Tree<base::Base<BaseJob>, merge::Merge<MergeJob>>>,
    /// last emitted proof and the corresponding transactions
    acc: Option<(MergeJob, Vec<BaseJob>)>,
    /// Sequence number for the jobs added every block
    curr_job_seq_no: SequenceNumber,
    /// transaction_capacity_log_2
    max_base_jobs: u64,
    delay: u64,
}

pub(super) enum ResetKind {
    Base,
    Merge,
    Both,
}

impl<BaseJob, MergeJob> Tree<base::Base<BaseJob>, merge::Merge<MergeJob>>
where
    BaseJob: Clone + Debug + 'static,
    MergeJob: Clone + Debug + 'static,
{
    fn update(
        &self,
        completed_jobs: &[Job<BaseJob, MergeJob>],
        update_level: u64,
        sequence_no: SequenceNumber,
        lens: WeightLens,
    ) -> Result<(Self, Option<MergeJob>), ()> {
        let add_merges = |jobs: &[Job<BaseJob, MergeJob>],
                          current_level: u64,
                          merge_job: merge::Merge<MergeJob>|
         -> Result<(merge::Merge<MergeJob>, Option<MergeJob>), ()> {
            use merge::{
                Job::{Empty, Full, Part},
                Record,
            };
            use Job::{Base, Merge};

            let weight = merge_job.weight;
            let m = merge_job.job;

            let (w1, w2) = &weight;
            let (left, right) = (*lens.get(w1), *lens.get(w2));

            // println!("current_level={} update_level={}", current_level, update_level);
            if update_level > 0 && current_level == update_level - 1 {
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

                match (jobs, m) {
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
            } else if update_level > 0 && (current_level < update_level - 1) {
                // Update the job count for all the level above
                match jobs {
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

        let add_bases = |jobs: &[Job<BaseJob, MergeJob>], base: base::Base<BaseJob>| {
            use base::Job::{Empty, Full};
            use Job::{Base, Merge};

            let w = base.weight;
            let d = base.job;

            let weight = lens.get(&w);

            // println!("add_bases jobs={:?} w={}", jobs.len(), weight);
            match (jobs, d) {
                ([], d) => Ok(base::Base { weight: w, job: d }),
                ([Base(d)], Empty) => {
                    let w = lens.set(&w, weight - 1);

                    Ok(base::Base {
                        weight: w,
                        job: Full(base::Record {
                            job: d.clone(),
                            seq_no: sequence_no.clone(),
                            state: JobStatus::Todo,
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
            &add_merges,
            &add_bases,
            &|merge| merge.weight.clone(),
            completed_jobs,
            update_level,
            &|(w1, w2), a: &[Job<BaseJob, MergeJob>]| {
                let l = *lens.get(&w1) as usize;
                let r = *lens.get(&w2) as usize;

                // println!("split l={} r={} len={}", l, r, a.len());

                (take(a, l), take_at(a, l, r))
            },
        )
    }

    fn reset_weights(&self, reset_kind: ResetKind) -> Self {
        let fun_base = |base: &base::Base<BaseJob>| {
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
                    update_merge_weight(weight),
                    set_zero(WeightLens::Merge, weight),
                ),
                ResetKind::Base => (
                    update_base_weight(weight),
                    set_zero(WeightLens::Base, weight),
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

        let fun_merge = |lst: ((Weight, Weight), (Weight, Weight)),
                         merge: &merge::Merge<MergeJob>| {
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

        let (result, _) = self.update_accumulate(&fun_merge, &fun_base);
        result
    }

    fn jobs_on_level(&self, depth: u64, level: u64) -> Vec<AvailableJob<BaseJob, MergeJob>> {
        use JobStatus::Todo;

        // self.iter()
        //     .filter(|(d, _)| *d == depth)
        //     .filter_map(|(_, value)| match value {
        //         Value::Leaf(base::Base {
        //             job:
        //                 base::Job::Full(base::Record {
        //                     job, state: Todo, ..
        //                 }),
        //             ..
        //         }) => Some(AvailableJob::Base(job.clone())),
        //         Value::Node(merge::Merge {
        //             job:
        //                 merge::Job::Full(merge::Record {
        //                     left,
        //                     right,
        //                     state: Todo,
        //                     ..
        //                 }),
        //             ..
        //         }) => Some(AvailableJob::Merge {
        //             left: left.clone(),
        //             right: right.clone(),
        //         }),
        //         _ => None,
        //     })
        //     .collect::<Vec<_>>();

        self.fold_depth(
            |i, mut acc, a| {
                use merge::{Job::Full, Record};

                if let (
                    true,
                    Full(Record {
                        left,
                        right,
                        state: Todo,
                        ..
                    }),
                ) = (i == level, &a.job)
                {
                    acc.push(AvailableJob::Merge {
                        left: left.clone(),
                        right: right.clone(),
                    });
                };
                acc
            },
            |mut acc, d| {
                use base::{Job::Full, Record};

                if let (
                    true,
                    Full(Record {
                        job, state: Todo, ..
                    }),
                ) = (level == depth, &d.job)
                {
                    acc.push(AvailableJob::Base(job.clone()));
                }
                acc
            },
            Vec::with_capacity(256),
        )
    }

    fn to_hashable_jobs(&self) -> Vec<Job<base::Base<BaseJob>, merge::Merge<MergeJob>>> {
        use JobStatus::Done;

        self.fold(
            |mut acc, a| {
                match &a.job {
                    merge::Job::Full(merge::Record { state: Done, .. }) => {}
                    _ => {
                        acc.push(Job::Merge(a.clone()));
                    }
                }
                acc
            },
            |mut acc, d| {
                match &d.job {
                    base::Job::Full(base::Record { state: Done, .. }) => {}
                    _ => {
                        acc.push(Job::Base(d.clone()));
                    }
                }
                acc
            },
            Vec::with_capacity(256),
        )
    }

    fn jobs_records(&self) -> Vec<Job<base::Record<BaseJob>, merge::Record<MergeJob>>> {
        self.fold(
            |mut acc, a: &merge::Merge<MergeJob>| {
                if let merge::Job::Full(x) = &a.job {
                    acc.push(Job::Merge(x.clone()));
                }
                acc
            },
            |mut acc, d: &base::Base<BaseJob>| {
                if let base::Job::Full(j) = &d.job {
                    acc.push(Job::Base(j.clone()));
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

    fn leaves(&self) -> Vec<base::Base<BaseJob>> {
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
        match &self.values[0] {
            Value::Node(value) => {
                let (w1, w2) = &value.weight;
                w1.merge + w2.merge
            }
            Value::Leaf(base) => base.weight.merge,
        }
    }

    fn available_space(&self) -> u64 {
        match &self.values[0] {
            Value::Node(value) => {
                let (w1, w2) = &value.weight;
                w1.base + w2.base
            }
            Value::Leaf(base) => base.weight.base,
        }
    }

    fn create_tree_for_level(
        level: i64,
        depth: u64,
        merge_job: merge::Job<MergeJob>,
        base_job: base::Job<BaseJob>,
    ) -> Self {
        let base_weight = if level == -1 {
            Weight::zero()
        } else {
            Weight { base: 1, merge: 0 }
        };

        let make_base = || base::Base {
            weight: base_weight.clone(),
            job: base_job.clone(),
        };

        let make_merge = |d: u64| {
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
        };

        let nnodes = 2u64.pow((depth + 1) as u32) - 1;

        let values: Vec<_> = (0..nnodes)
            .into_iter()
            .map(|index| {
                let node_depth = btree::depth_at(index as usize);

                if node_depth == depth {
                    Value::Leaf(make_base())
                } else {
                    Value::Node(make_merge(node_depth))
                }
            })
            .collect();

        // println!("first={:?}", values[0]);

        // println!("nnodes={:?} len={:?} nbases={:?}", nnodes, values.len(), values.iter().filter(|v| {
        //   matches!(v, Value::Leaf(_))
        // }).count());

        Self { values }
    }

    fn create_tree(depth: u64) -> Self {
        let level: i64 = depth.try_into().unwrap();
        Self::create_tree_for_level(level, depth, merge::Job::Empty, base::Job::Empty)
    }
}

impl<BaseJob, MergeJob> ParallelScan<BaseJob, MergeJob>
where
    BaseJob: Debug + Clone + 'static,
    MergeJob: Debug + Clone + 'static,
{
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

    pub fn empty(max_base_jobs: u64, delay: u64) -> Self {
        let depth = ceil_log2(max_base_jobs);
        // println!("empty depth={:?}", depth);

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
            .map(|tree| tree.map_depth(&|_, m| m.map(&f1), &|a| a.map(&f2)))
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

    pub fn hash<FunMerge, FunBase>(
        &self,
        fun_merge: FunMerge,
        fun_base: FunBase,
    ) -> GenericArray<u8, U32>
    where
        FunMerge: Fn(&mut Vec<u8>, &MergeJob),
        FunBase: Fn(&mut Vec<u8>, &BaseJob),
    {
        const BUFFER_CAPACITY: usize = 128 * 1024;

        let Self {
            trees,
            acc,
            curr_job_seq_no,
            max_base_jobs,
            delay,
        } = self.with_leaner_trees();

        let mut sha: Sha256 = Sha256::new();
        let mut buffer = Vec::with_capacity(BUFFER_CAPACITY);
        let buffer = &mut buffer;

        let add_weight =
            |buffer: &mut Vec<u8>, w: &Weight| write!(buffer, "{}{}", w.base, w.merge).unwrap();

        let add_two_weights = |buffer: &mut Vec<u8>, (w1, w2): &(Weight, Weight)| {
            add_weight(buffer, w1);
            add_weight(buffer, w2);
        };

        for job in trees.iter().flat_map(Tree::to_hashable_jobs) {
            buffer.clear();

            match &job {
                Job::Base(base) => match &base.job {
                    base::Job::Empty => {
                        add_weight(buffer, &base.weight);
                        write!(buffer, "Empty").unwrap();
                    }
                    base::Job::Full(base::Record { job, seq_no, state }) => {
                        add_weight(buffer, &base.weight);
                        write!(buffer, "Full{}{}", seq_no.0, state.as_str()).unwrap();

                        fun_base(buffer, job);
                    }
                },
                Job::Merge(merge) => match &merge.job {
                    merge::Job::Empty => {
                        add_two_weights(buffer, &merge.weight);
                        write!(buffer, "Empty").unwrap();
                    }
                    merge::Job::Part(job) => {
                        add_two_weights(buffer, &merge.weight);
                        write!(buffer, "Part").unwrap();

                        fun_merge(buffer, job);
                    }
                    merge::Job::Full(merge::Record {
                        left,
                        right,
                        seq_no,
                        state,
                    }) => {
                        add_two_weights(buffer, &merge.weight);
                        write!(buffer, "Full{}{}", seq_no.0, state.as_str()).unwrap();

                        fun_merge(buffer, left);
                        fun_merge(buffer, right);
                    }
                },
            }

            sha.update(buffer.as_slice());

            // TODO: Remove this assert once we know it's a good capacity
            //       (buffer is not resized for serialization)
            assert_eq!(buffer.capacity(), BUFFER_CAPACITY);
        }

        match &acc {
            Some((a, d_lst)) => {
                buffer.clear();

                fun_merge(buffer, a);
                for j in d_lst {
                    fun_base(buffer, j);
                }

                sha.update(&buffer);
            }
            None => {
                sha.update("None");
            }
        };

        buffer.clear();
        write!(buffer, "{}{}{}", curr_job_seq_no.0, max_base_jobs, delay).unwrap();
        sha.update(&buffer);

        sha.finalize()
    }

    fn fold_chronological_until<Accum, Final, FunMerge, FunBase, FunFinish>(
        &self,
        init: Accum,
        fun_merge: FunMerge,
        fun_base: FunBase,
        fun_finish: FunFinish,
    ) -> Final
    where
        FunMerge: Fn(Accum, &merge::Merge<MergeJob>) -> ControlFlow<Final, Accum>,
        FunBase: Fn(Accum, &base::Base<BaseJob>) -> ControlFlow<Final, Accum>,
        FunFinish: Fn(Accum) -> Final,
    {
        let mut accum = init;

        for tree in self.trees.iter().rev() {
            match tree.fold_depth_until_prime(&|_, acc, m| fun_merge(acc, m), &fun_base, accum) {
                Continue(acc) => accum = acc,
                Break(v) => return v,
            }
        }

        fun_finish(accum)
    }

    fn fold_chronological<Accum, FunMerge, FunBase>(
        &self,
        init: Accum,
        fun_merge: FunMerge,
        fun_base: FunBase,
    ) -> Accum
    where
        FunMerge: Fn(Accum, &merge::Merge<MergeJob>) -> Accum,
        FunBase: Fn(Accum, &base::Base<BaseJob>) -> Accum,
    {
        self.fold_chronological_until(
            init,
            |acc, a| Continue(fun_merge(acc, a)),
            |acc, d| Continue(fun_base(acc, d)),
            |v| v,
        )
    }

    fn max_trees(&self) -> u64 {
        ((ceil_log2(self.max_base_jobs) + 1) * (self.delay + 1)) + 1
    }

    fn work_for_tree(&self, data_tree: WorkForTree) -> Vec<AvailableJob<BaseJob, MergeJob>> {
        let delay = self.delay + 1;

        // TODO: Not sure if skip(1) is correct below
        let trees = match data_tree {
            WorkForTree::Current => &self.trees[1..],
            WorkForTree::Next => &self.trees,
        };

        // println!("WORK_FOR_TREE len={} delay={}", trees.len(), delay);

        work(trees, delay, self.max_base_jobs)
    }

    /// work on all the level and all the trees
    fn all_work(&self) -> Vec<Vec<AvailableJob<BaseJob, MergeJob>>> {
        let depth = ceil_log2(self.max_base_jobs);
        // TODO: Not sure if it's correct
        let set1 = self.work_for_tree(WorkForTree::Current);
        // let setaaa = self.work_for_tree(WorkForTree::Next);

        // println!(
        //     "ntrees={} delay={} set1={}",
        //     self.trees.len(),
        //     self.delay,
        //     set1.len()
        // );
        // println!("ntrees={} set1={} setaa={}", self.trees.len(), set1.len(), setaaa.len());

        let mut this = self.clone();
        this.trees.reserve(self.delay as usize + 1);

        // println!("set1={:?}", set1.len());

        let mut other_set = Vec::with_capacity(256);
        other_set.push(set1);

        for _ in 0..self.delay + 1 {
            // println!("trees={}", this.trees.len());
            this.trees.insert(0, Tree::create_tree(depth));
            let work = this.work_for_tree(WorkForTree::Current);

            // println!("work={}", work.len());

            if !work.is_empty() {
                other_set.push(work);
            }
        }

        other_set
    }

    // let all_work :
    //     type merge base. (merge, base) t -> (merge, base) Available_job.t list list
    //     =
    //  fun t ->
    //   let depth = Int.ceil_log2 t.max_base_jobs in
    //   Printf.eprintf "ntrees=%d\n%!" (Non_empty_list.length t.trees);
    //   let set1 = work_for_tree t ~data_tree:`Current in
    //   let _, other_sets =
    //     List.fold ~init:(t, [])
    //       (List.init ~f:Fn.id (t.delay + 1))
    //       ~f:(fun (t, work_list) _ ->
    //         Printf.eprintf "trees=%d\n%!" (Non_empty_list.length t.trees);
    //         let trees' = Non_empty_list.cons (create_tree ~depth) t.trees in
    //         let t' = { t with trees = trees' } in
    //         let work = work_for_tree t' ~data_tree:`Current in
    //         Printf.eprintf "work=%d\n%!" (List.length work);
    //         match work_for_tree t' ~data_tree:`Current with
    //         | [] ->
    //             (t', work_list)
    //         | work ->
    //             (t', work :: work_list) )
    //   in
    //   Printf.eprintf "set1=%d\n%!" (List.length set1);
    //   if List.is_empty set1 then List.rev other_sets
    //   else set1 :: List.rev other_sets

    fn work_for_next_update(&self, data_count: u64) -> Vec<Vec<AvailableJob<BaseJob, MergeJob>>> {
        let delay = self.delay + 1;
        let current_tree_space = self.trees[0].available_space() as usize;

        let mut set1 = work(&self.trees[1..], delay, self.max_base_jobs);
        let count = data_count.min(self.max_base_jobs) as usize;

        if current_tree_space < count {
            let mut set2 = work(&self.trees, delay, self.max_base_jobs);
            set2.truncate((count - current_tree_space) * 2);

            vec![set1, set2]
                .into_iter()
                .filter(|v| !v.is_empty())
                .collect()
        } else {
            set1.truncate(2 * count);

            if set1.is_empty() {
                vec![]
            } else {
                vec![set1]
            }
        }
    }

    fn free_space_on_current_tree(&self) -> u64 {
        self.trees[0].available_space()
    }

    fn add_merge_jobs(
        &mut self,
        completed_jobs: &[MergeJob],
    ) -> Result<Option<(MergeJob, Vec<BaseJob>)>, ()> {
        fn take<T>(slice: &[T], n: usize) -> &[T] {
            slice.get(..n).unwrap_or(slice)
        }

        fn drop<T>(slice: &[T], n: usize) -> &[T] {
            slice.get(n..).unwrap_or(&[])
        }

        if completed_jobs.is_empty() {
            return Ok(None);
        }

        let delay = self.delay + 1;
        let udelay = delay as usize;
        let depth = ceil_log2(self.max_base_jobs);

        let completed_jobs_len = completed_jobs.len();
        let merge_jobs: Vec<_> = completed_jobs.iter().cloned().map(Job::Merge).collect();
        let jobs_required = self.work_for_tree(WorkForTree::Current);

        assert!(
            merge_jobs.len() <= jobs_required.len(),
            "More work than required"
        );

        let curr_tree = &self.trees[0];
        let to_be_updated_trees = &self.trees[1..];

        let (mut updated_trees, result_opt) = {
            let mut jobs = merge_jobs.as_slice();

            let mut updated_trees = Vec::with_capacity(to_be_updated_trees.len());
            let mut scan_result = None;

            for (i, tree) in to_be_updated_trees.iter().enumerate() {
                // Every nth (n=delay) tree
                if (i % udelay == udelay - 1) && !jobs.is_empty() {
                    let nrequired = tree.required_job_count() as usize;
                    let completed_jobs = take(jobs, nrequired);
                    let i = i as u64;

                    let (tree, result) = tree.update(
                        completed_jobs,
                        depth - (i / delay),
                        self.curr_job_seq_no.clone(),
                        WeightLens::Merge,
                    )?;

                    updated_trees.push(tree);
                    scan_result = result;
                    jobs = drop(jobs, nrequired);
                } else {
                    updated_trees.push(tree.clone());
                }
            }

            (updated_trees, scan_result)
        };

        let (mut updated_trees, result_opt) = {
            let (updated_trees, result_opt) = match result_opt {
                Some(scan_result) if !updated_trees.is_empty() => {
                    let last = updated_trees.pop().unwrap();
                    let tree_data = last.base_jobs();
                    (updated_trees, Some((scan_result, tree_data)))
                }
                _ => (updated_trees, None),
            };

            // TODO: Not sure if priority is same as OCaml here
            if result_opt.is_some()
                || (updated_trees.len() + 1) < self.max_trees() as usize
                    && (completed_jobs_len == jobs_required.len())
            {
                let updated_trees = updated_trees
                    .into_iter()
                    .map(|tree| tree.reset_weights(ResetKind::Merge))
                    .collect();
                (updated_trees, result_opt)
            } else {
                (updated_trees, result_opt)
            }
        };

        updated_trees.insert(0, curr_tree.clone());

        self.trees = updated_trees;

        Ok(result_opt)
    }

    fn add_data(&mut self, data: Vec<BaseJob>) -> Result<(), ()> {
        if data.is_empty() {
            return Ok(());
        }

        let data_len = data.len();
        let depth = ceil_log2(self.max_base_jobs);
        let tree = &self.trees[0];

        let base_jobs: Vec<_> = data.into_iter().map(Job::Base).collect();
        let available_space = tree.available_space() as usize;

        assert!(
            data_len <= available_space,
            "Data count ({}) exceeded available space ({})",
            data_len,
            available_space
        );

        // println!(
        //     "base_jobs={:?} available_space={:?} depth={:?}",
        //     base_jobs.len(),
        //     available_space,
        //     depth
        // );

        let (tree, _) = tree
            .update(
                &base_jobs,
                depth,
                self.curr_job_seq_no.clone(),
                WeightLens::Base,
            )
            .expect("Error while adding a base job to the tree");

        let updated_trees = if data_len == available_space {
            let new_tree = Tree::create_tree(depth);
            let tree = tree.reset_weights(ResetKind::Both);
            vec![new_tree, tree]
        } else {
            let tree = tree.reset_weights(ResetKind::Merge);
            vec![tree]
        };

        // println!(
        //     "updated_trees={} self_trees={:?}",
        //     updated_trees.len(),
        //     self.trees.len()
        // );

        // println!("WORK1={:?}", work(updated_trees.as_slice(), self.delay + 1, self.max_base_jobs));
        // println!("WORK2={:?}", work(self.trees.as_slice(), self.delay + 1, self.max_base_jobs));

        // self.trees.append(&mut updated_trees);

        // let tail = &self.trees[1..];
        // self.trees = updated_trees.into_iter().zip(tail.iter().cloned().collect()).collect();

        let mut old = std::mem::replace(&mut self.trees, updated_trees);
        old.remove(0);
        self.trees.append(&mut old);

        // println!("WORK3={:?}", work(self.trees.as_slice(), self.delay + 1, self.max_base_jobs));

        Ok(())
    }

    fn reset_seq_no(&mut self) {
        let last = self.trees.last().unwrap();
        let oldest_seq_no = match last.leaves().first() {
            Some(base::Base {
                job: base::Job::Full(base::Record { seq_no, .. }),
                ..
            }) => seq_no.clone(),
            _ => SequenceNumber::zero(),
        };

        let new_seq = |seq: &SequenceNumber| (seq - &oldest_seq_no).incr();

        let fun_merge = |m: &merge::Merge<MergeJob>| match &m.job {
            merge::Job::Full(merge::Record { seq_no, .. }) => m.with_seq_no(new_seq(seq_no)),
            _ => m.clone(),
        };

        let fun_base = |m: &base::Base<BaseJob>| match &m.job {
            base::Job::Full(base::Record { seq_no, .. }) => m.with_seq_no(new_seq(seq_no)),
            _ => m.clone(),
        };

        let mut max_seq = SequenceNumber::zero();
        let mut updated_trees = Vec::with_capacity(self.trees.len());

        for tree in &self.trees {
            use base::{Base, Job::Full, Record};

            let tree = tree.map(fun_merge, fun_base);
            updated_trees.push(tree.clone());

            let leaves = tree.leaves();

            let last = match leaves.last() {
                Some(last) => last,
                None => continue,
            };

            if let Base {
                job: Full(Record { seq_no, .. }),
                ..
            } = last
            {
                max_seq = max_seq.max(seq_no.clone());
            };
        }

        self.curr_job_seq_no = max_seq;
        self.trees = updated_trees;
    }

    fn incr_sequence_no(&mut self) {
        let next_seq_no = self.curr_job_seq_no.incr();

        if next_seq_no.is_u64_max() {
            self.reset_seq_no();
        } else {
            self.curr_job_seq_no = next_seq_no;
        }
    }

    fn update_helper(
        &mut self,
        data: Vec<BaseJob>,
        completed_jobs: Vec<MergeJob>,
    ) -> Result<Option<(MergeJob, Vec<BaseJob>)>, ()> {
        fn split<T>(slice: &[T], n: usize) -> (&[T], &[T]) {
            (
                slice.get(..n).unwrap_or(slice),
                slice.get(n..).unwrap_or(&[]),
            )
        }

        let data_count = data.len() as u64;

        assert!(
            data_count <= self.max_base_jobs,
            "Data count ({}) exceeded maximum ({})",
            data_count,
            self.max_base_jobs
        );

        let required_jobs_count = self
            .work_for_next_update(data_count)
            .into_iter()
            .flatten()
            .count();

        {
            let required = (required_jobs_count + 1) / 2;
            let got = (completed_jobs.len() + 1) / 2;

            // println!("required={:?} got={:?}", required, got);

            let max_base_jobs = self.max_base_jobs as usize;
            assert!(
                !(got < required && data.len() > max_base_jobs - required + got),
                "Insufficient jobs (Data count {}): Required- {} got- {}",
                data_count,
                required,
                got
            )
        }

        let delay = self.delay + 1;

        // Increment the sequence number
        self.incr_sequence_no();

        let latest_tree = &self.trees[0];
        let available_space = latest_tree.available_space();

        // Possible that new base jobs is added to a new tree within an
        // update i.e., part of it is added to the first tree and the rest
        // of it to a new tree. This happens when the throughput is not max.
        // This also requires merge jobs to be done on two different set of trees*)

        let (data1, data2) = split(&data, available_space as usize);

        // println!(
        //     "delay={} available_space={} data1={} data2={}",
        //     delay,
        //     available_space,
        //     data1.len(),
        //     data2.len()
        // );

        let required_jobs_for_current_tree =
            work(&self.trees[1..], delay, self.max_base_jobs).len();
        let (jobs1, jobs2) = split(&completed_jobs, required_jobs_for_current_tree);

        // println!(
        //     "required_jobs_for_current_tree={} jobs1={} jobs2={}",
        //     required_jobs_for_current_tree,
        //     jobs1.len(),
        //     jobs2.len()
        // );

        // update first set of jobs and data
        let result_opt = self.add_merge_jobs(jobs1)?;
        self.add_data(data1.to_vec())?;

        // update second set of jobs and data.
        // This will be empty if all the data fit in the current tree
        self.add_merge_jobs(jobs2)?;
        self.add_data(data2.to_vec())?;

        // update the latest emitted value
        if result_opt.is_some() {
            self.acc = result_opt.clone();
        };

        assert!(
            self.trees.len() <= self.max_trees() as usize,
            "Tree list length ({}) exceeded maximum ({})",
            self.trees.len(),
            self.max_trees()
        );

        Ok(result_opt)
    }

    fn update(
        &mut self,
        data: Vec<BaseJob>,
        completed_jobs: Vec<MergeJob>,
    ) -> Result<Option<(MergeJob, Vec<BaseJob>)>, ()> {
        self.update_helper(data, completed_jobs)
    }

    fn all_jobs(&self) -> Vec<Vec<AvailableJob<BaseJob, MergeJob>>> {
        self.all_work()
    }

    fn jobs_for_next_update(&self) -> Vec<Vec<AvailableJob<BaseJob, MergeJob>>> {
        self.work_for_next_update(self.max_base_jobs)
    }

    fn jobs_for_slots(&self, slots: u64) -> Vec<Vec<AvailableJob<BaseJob, MergeJob>>> {
        self.work_for_next_update(slots)
    }

    fn free_space(&self) -> u64 {
        self.max_base_jobs
    }

    fn last_emitted_value(&self) -> Option<&(MergeJob, Vec<BaseJob>)> {
        self.acc.as_ref()
    }

    fn current_job_sequence_number(&self) -> SequenceNumber {
        self.curr_job_seq_no.clone()
    }

    fn base_jobs_on_latest_tree(&self) -> Vec<AvailableJob<BaseJob, MergeJob>> {
        let depth = ceil_log2(self.max_base_jobs);
        let level = depth;

        self.trees[0]
            .jobs_on_level(depth, level)
            .into_iter()
            .filter(|job| matches!(job, AvailableJob::Base(_)))
            .collect()
    }

    // 0-based indexing, so 0 indicates next-to-latest tree
    fn base_jobs_on_earlier_tree(&self, index: usize) -> Vec<AvailableJob<BaseJob, MergeJob>> {
        let depth = ceil_log2(self.max_base_jobs);
        let level = depth;

        let earlier_trees = &self.trees[1..];

        match earlier_trees.get(index) {
            None => vec![],
            Some(tree) => tree
                .jobs_on_level(depth, level)
                .into_iter()
                .filter(|job| matches!(job, AvailableJob::Base(_)))
                .collect(),
        }
    }

    fn partition_if_overflowing(&self) -> SpacePartition {
        let cur_tree_space = self.free_space_on_current_tree();

        // Check actual work count because it would be zero initially

        let work_count = self.work_for_tree(WorkForTree::Current).len() as u64;
        let work_count_new_tree = self.work_for_tree(WorkForTree::Next).len() as u64;

        SpacePartition {
            first: (cur_tree_space, work_count),
            second: {
                if cur_tree_space < self.max_base_jobs {
                    let slots = self.max_base_jobs - cur_tree_space;
                    Some((slots, work_count_new_tree.min(2 * slots)))
                } else {
                    None
                }
            },
        }
    }

    fn next_on_new_tree(&self) -> bool {
        let curr_tree_space = self.free_space_on_current_tree();
        curr_tree_space == self.max_base_jobs
    }

    fn pending_data(&self) -> Vec<BaseJob> {
        self.trees.iter().rev().flat_map(Tree::base_jobs).collect()
    }

    // #[cfg(test)]
    fn job_count(&self) -> (f64, f64) {
        use JobStatus::{Done, Todo};

        self.fold_chronological(
            (0.0, 0.0),
            |(ntodo, ndone), merge| {
                use merge::{
                    Job::{Empty, Full, Part},
                    Record,
                };

                let (todo, done) = match &merge.job {
                    Part(_) => (0.5, 0.0),
                    Full(Record { state: Todo, .. }) => (1.0, 0.0),
                    Full(Record { state: Done, .. }) => (0.0, 1.0),
                    Empty => (0.0, 0.0),
                };
                (ntodo + todo, ndone + done)
            },
            |(ntodo, ndone), base| {
                use base::{
                    Job::{Empty, Full},
                    Record,
                };

                let (todo, done) = match &base.job {
                    Empty => (0.0, 0.0),
                    Full(Record { state: Todo, .. }) => (1.0, 0.0),
                    Full(Record { state: Done, .. }) => (0.0, 1.0),
                };

                (ntodo + todo, ndone + done)
            },
        )
    }
}

fn work_to_do<'a, BaseJob, MergeJob, I>(
    trees: I,
    max_base_jobs: u64,
) -> Vec<AvailableJob<BaseJob, MergeJob>>
where
    I: Iterator<Item = &'a Tree<base::Base<BaseJob>, merge::Merge<MergeJob>>>,
    BaseJob: Debug + Clone + 'static,
    MergeJob: Debug + Clone + 'static,
{
    let depth = ceil_log2(max_base_jobs);

    let trees: Vec<_> = trees.collect();

    // println!("work_to_do length={}", trees.len());

    trees
        .iter()
        .enumerate()
        .flat_map(|(i, tree)| {
            let level = depth - i as u64;
            tree.jobs_on_level(depth, level)
        })
        .collect()
}

fn work<'a, BaseJob, MergeJob, I>(
    trees: I,
    delay: u64,
    max_base_jobs: u64,
) -> Vec<AvailableJob<BaseJob, MergeJob>>
where
    I: IntoIterator<Item = &'a Tree<base::Base<BaseJob>, merge::Merge<MergeJob>>>,
    BaseJob: Debug + Clone + 'static,
    MergeJob: Debug + Clone + 'static,
{
    let depth = ceil_log2(max_base_jobs) as usize;
    let delay = delay as usize;

    // println!("WORK_DELAY={}", delay);

    let work_trees = trees
        .into_iter()
        .enumerate()
        .filter_map(|(i, t)| {
            if i % delay == delay - 1 {
                Some(t)
            } else {
                None
            }
        })
        .take(depth + 1);

    work_to_do(work_trees, max_base_jobs)
}

fn take<T>(slice: &[T], n: usize) -> &[T] {
    slice.get(..n).unwrap_or(slice)
}

fn take_at<T>(slice: &[T], skip: usize, n: usize) -> &[T] {
    slice.get(skip..).map(|s| take(s, n)).unwrap_or(&[])
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

fn flatten<T>(v: Vec<Vec<T>>) -> Vec<T> {
    v.into_iter().flatten().collect()
}

// #[cfg(test)]
fn assert_job_count<B, M>(
    s1: &ParallelScan<B, M>,
    s2: &ParallelScan<B, M>,
    completed_job_count: f64,
    base_job_count: f64,
    value_emitted: bool,
) where
    B: Debug + Clone + 'static,
    M: Debug + Clone + 'static,
{
    // println!("s1={:#?}", s1);
    // println!("s2={:?}", s2);

    let (todo_before, done_before) = s1.job_count();
    let (todo_after, done_after) = s2.job_count();

    // println!(
    //     "before todo={:?} done={:?}",
    //     s1.job_count().0,
    //     s1.job_count().1
    // );
    // println!(
    //     "after  todo={:?} done={:?}",
    //     s2.job_count().0,
    //     s2.job_count().1
    // );

    // ordered list of jobs that is actually called when distributing work
    let all_jobs = flatten(s2.all_jobs());

    // list of jobs

    let all_jobs_expected_count = s2
        .trees
        .iter()
        .fold(Vec::with_capacity(s2.trees.len()), |mut acc, tree| {
            let mut records = tree.jobs_records();
            acc.append(&mut records);
            acc
        })
        .into_iter()
        .filter(|job| match job {
            Job::Base(base::Record {
                state: JobStatus::Todo,
                ..
            }) => true,
            Job::Merge(merge::Record {
                state: JobStatus::Todo,
                ..
            }) => true,
            _ => false,
        })
        .count();

    // println!(
    //     "all_jobs={} all_jobs_expected={}",
    //     all_jobs.len(),
    //     all_jobs_expected_count
    // );

    assert_eq!(all_jobs.len(), all_jobs_expected_count);

    let expected_todo_after = {
        let new_jobs = if value_emitted {
            (completed_job_count - 1.0) / 2.0
        } else {
            completed_job_count / 2.0
        };
        todo_before + base_job_count - completed_job_count + new_jobs
    };

    let expected_done_after = {
        let jobs_from_delete_tree = if value_emitted {
            ((2 * s1.max_base_jobs) - 1) as f64
        } else {
            0.0
        };
        done_before + completed_job_count - jobs_from_delete_tree
    };

    assert_eq!(todo_after, expected_todo_after);
    assert_eq!(done_after, expected_done_after);
}

fn test_update<B, M>(
    s1: &ParallelScan<B, M>,
    data: Vec<B>,
    completed_jobs: Vec<M>,
) -> (Option<(M, Vec<B>)>, ParallelScan<B, M>)
where
    B: Debug + Clone + 'static,
    M: Debug + Clone + 'static,
{
    let mut s2 = s1.clone();
    let result_opt = s2.update(data.clone(), completed_jobs.clone()).unwrap();

    assert_job_count(
        s1,
        &s2,
        completed_jobs.len() as f64,
        data.len() as f64,
        result_opt.is_some(),
    );
    (result_opt, s2)
}

fn int_to_string(u: &usize) -> String {
    u.to_string()
}

// fn sint_to_string(i: &i64) -> String {
//     i.to_string()
// }

fn sint_to_string(buffer: &mut Vec<u8>, i: &i64) {
    write!(buffer, "{}", i).unwrap();
}

fn hash(state: &ParallelScan<i64, i64>) -> String {
    hex::encode(state.hash(sint_to_string, sint_to_string))
}

#[cfg(test)]
mod tests {
    use std::{
        array,
        sync::{
            atomic::{AtomicBool, Ordering::Relaxed},
            mpsc::{sync_channel, Receiver, SyncSender},
            Arc,
        },
    };

    use rand::Rng;

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

    // Make sure that sha256 produces same result when data is splitted or not
    #[test]
    fn test_sha256() {
        let array: [u8; 2 * 1024] = array::from_fn(|i| (i % 256) as u8);
        let mut slice = &array[..];

        let mut sha256 = sha2::Sha256::new();
        for byte in slice.iter().copied() {
            sha256.update(&[byte][..]);
        }
        let first = sha256.finalize();

        let mut sha256 = sha2::Sha256::new();
        let mut n = 1;
        while !slice.is_empty() {
            sha256.update(slice.get(..n).unwrap_or(slice));
            slice = slice.get(n..).unwrap_or(&[]);

            n += 2;
        }
        let second = sha256.finalize();

        assert_eq!(first, second);
    }

    #[test]
    fn test_range_at_depth() {
        let ranges: Vec<_> = (0..10u64).map(btree::range_at_depth).collect();

        assert_eq!(
            ranges,
            [
                0..1,
                1..3,
                3..7,
                7..15,
                15..31,
                31..63,
                63..127,
                127..255,
                255..511,
                511..1023,
            ]
        );
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/parallel_scan/parallel_scan.ml#L1525
    #[test]
    fn always_max_base_jobs() {
        const MAX_BASE_JOS: u64 = 512;

        // let int_to_string = |i: &usize| i.to_string();

        // let state = ParallelScan::<usize, usize>::empty(8, 3);
        // println!("STATE len={:?} ={:#?}", state.trees.len(), state);
        // println!("hash={:?}", hash(&state));

        let mut state = ParallelScan::<usize, usize>::empty(MAX_BASE_JOS, 3);
        let mut expected_result: Vec<Vec<usize>> = vec![];

        // println!("hash={:?}", hex::encode(state.hash(int_to_string, int_to_string)));
        // println!("hash={:?}", state.hash(int_to_string, int_to_string));

        for i in 0..100 {
            println!("####### LOOP {:?} #########", i);

            let data: Vec<_> = (0..MAX_BASE_JOS as usize)
                .into_iter()
                .map(|j| i + j)
                .collect();

            expected_result.push(data.clone());

            let work: Vec<_> = state
                .work_for_next_update(data.len() as u64)
                .into_iter()
                .flatten()
                .collect();

            let new_merges: Vec<_> = work
                .iter()
                .map(|job| match job {
                    AvailableJob::Base(i) => *i,
                    AvailableJob::Merge { left, right } => *left + *right,
                })
                .collect();

            println!("work={:?} new_merges={:?}", work.len(), new_merges.len());
            // println!("hash_s1={:?}", hash(&state));

            // let mut s2 = state.clone();
            // let result_opt = s2.update(data.clone(), new_merges.clone()).unwrap();
            // println!("hash_s2={:?}", hash(&s2));

            let (result_opt, s) = test_update(&state, data, new_merges);

            // assert!(result_opt.is_none());

            let (expected_result_, remaining_expected_results) = {
                match result_opt {
                    None => ((0, vec![]), expected_result.clone()),
                    Some(ref r) => {
                        println!("RESULT_OPT.0={} len={}", r.0, r.1.len());
                        // println!("expected_result={:?}", expected_result);
                        // Printf.eprintf "RESULT_OPT.0=%d len=%d\n%!" a (List.length l);
                        if expected_result.is_empty() {
                            ((0, vec![]), vec![])
                        } else {
                            let first = expected_result[0].clone();
                            let sum: usize = first.iter().sum();

                            ((sum, first), expected_result[1..].to_vec())
                        }
                    }
                }
            };

            assert_eq!(
                result_opt.as_ref().unwrap_or(&expected_result_),
                &expected_result_
            );

            expected_result = remaining_expected_results;
            state = s;
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/parallel_scan/parallel_scan.ml#L1562
    #[test]
    fn random_base_jobs() {
        const MAX_BASE_JOS: usize = 512;

        let mut state = ParallelScan::<usize, usize>::empty(MAX_BASE_JOS as u64, 3);
        let mut rng = rand::thread_rng();
        let expected_result = (MAX_BASE_JOS, vec![1usize; MAX_BASE_JOS]);

        for _ in 0..1_000 {
            let mut data = vec![1; rng.gen_range(0..=30)];
            data.truncate(MAX_BASE_JOS);
            let data_len = data.len();

            println!("list_length={}", data_len);

            let work: Vec<_> = state
                .work_for_next_update(data_len as u64)
                .into_iter()
                .flatten()
                .take(data_len * 2)
                .collect();
            let new_merges: Vec<_> = work
                .iter()
                .map(|job| match job {
                    AvailableJob::Base(i) => *i,
                    AvailableJob::Merge { left, right } => left + right,
                })
                .collect();

            let (result_opt, s) = test_update(&state, data, new_merges);

            assert_eq!(
                result_opt.as_ref().unwrap_or(&expected_result),
                &expected_result
            );

            state = s;
        }
    }

    fn gen<FunDone, FunAcc>(fun_job_done: FunDone, fun_acc: FunAcc) -> ParallelScan<i64, i64>
    where
        FunDone: Fn(&AvailableJob<i64, i64>) -> i64,
        FunAcc: Fn(Option<(i64, Vec<i64>)>, (i64, Vec<i64>)) -> Option<(i64, Vec<i64>)>,
    {
        let mut rng = rand::thread_rng();

        let depth = rng.gen_range(2..5);
        let delay = rng.gen_range(0..=3);

        let max_base_jobs = 2u64.pow(depth);

        let mut s = ParallelScan::<i64, i64>::empty(max_base_jobs, delay);

        let ndatas = rng.gen_range(2..=20);
        let datas: Vec<Vec<i64>> = (1..ndatas)
            .map(|_| {
                std::iter::repeat_with(|| rng.gen())
                    .take(max_base_jobs as usize)
                    .collect()
            })
            .collect();

        // let datas = vec![
        //     // vec![-58823712978749242i64 as i64, 25103, 33363641, -1611555993190i64 as i64]
        //     // std::iter::repeat_with(|| rng.gen_range(0..1000))
        //     std::iter::repeat_with(|| rng.gen())
        //     // std::iter::repeat_with(|| 1)
        //         .take(max_base_jobs as usize)
        //         .collect::<Vec<_>>()
        // ];

        for data in datas {
            println!("ndata={}", data.len());

            let jobs = flatten(s.work_for_next_update(data.len() as u64));

            let jobs_done: Vec<i64> = jobs.iter().map(&fun_job_done).collect();

            println!("jobs_donea={}", jobs_done.len());

            let old_tuple = s.acc.clone();

            let res_opt = s.update(data, jobs_done).unwrap();

            if let Some(res) = res_opt {
                let tuple = if old_tuple.is_some() {
                    old_tuple
                } else {
                    s.acc.clone()
                };

                s.acc = fun_acc(tuple, res);
            }
            println!("s.acc.is_some={:?}", s.acc.is_some());
        }

        s
    }

    fn fun_merge_up(
        state: Option<(i64, Vec<i64>)>,
        mut x: (i64, Vec<i64>),
    ) -> Option<(i64, Vec<i64>)> {
        let mut acc = state?;
        acc.1.append(&mut x.1);
        Some((acc.0.wrapping_add(x.0), acc.1))
    }

    fn job_done(job: &AvailableJob<i64, i64>) -> i64 {
        match job {
            AvailableJob::Base(x) => *x,
            // AvailableJob::Merge { left, right } => left + right,
            AvailableJob::Merge { left, right } => {
                // let left = *left as i64;
                // let right = *right as i64;
                left.wrapping_add(*right)
                // left + right
            }
        }
    }

    /// scan (+) over ints
    ///
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/parallel_scan/parallel_scan.ml#L1677
    #[test]
    fn split_on_if_enqueuing_onto_the_next_queue() {
        let mut rng = rand::thread_rng();

        let p = 4;
        let max_base_jobs = 2u64.pow(p);

        for _ in 0..100000 {
            let state = ParallelScan::<i64, i64>::empty(max_base_jobs, 1);
            println!("hash_state={:?}", hash(&state));
            let i = rng.gen_range(0..max_base_jobs);

            let data: Vec<i64> = (0..i as i64).collect();

            let partition = state.partition_if_overflowing();
            let jobs = flatten(state.work_for_next_update(data.len() as u64));

            let jobs_done: Vec<i64> = jobs.iter().map(job_done).collect();

            let tree_count_before = state.trees.len();

            let (_, s) = test_update(&state, data, jobs_done);

            println!("second={:?}", partition.second.is_some());

            match partition.second {
                None => {
                    let tree_count_after = s.trees.len();
                    let expected_tree_count = if partition.first.0 == i {
                        tree_count_before + 1
                    } else {
                        tree_count_before
                    };
                    assert_eq!(tree_count_after, expected_tree_count);
                }
                Some(_) => {
                    let tree_count_after = s.trees.len();
                    let expected_tree_count = if i > partition.first.0 {
                        tree_count_before + 1
                    } else {
                        tree_count_before
                    };
                    assert_eq!(tree_count_after, expected_tree_count);
                }
            }
        }
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/parallel_scan/parallel_scan.ml#L1722
    #[test]
    fn sequence_number_reset() {
        let p = 3;
        let max_base_jobs = 2u64.pow(p);

        let jobs = |state: &ParallelScan<i64, i64>| -> Vec<Vec<Job<base::Record<i64>, merge::Record<i64>>>> {
            state.trees.iter().map(|t| t.jobs_records()).rev().collect()
        };

        let verify_sequence_number = |state: &ParallelScan<i64, i64>| {
            let mut state = state.clone();
            state.reset_seq_no();

            let jobs_list = jobs(&state);

            let depth = ceil_log2(max_base_jobs + 1);

            for (i, jobs) in jobs_list.iter().enumerate() {
                // each tree has jobs up till a level below the older tree
                //  and have the following sequence numbers after reset
                //             4
                //         3       3
                //       2   2   2   2
                //      1 1 1 1 1 1 1 1

                let cur_levels = depth - i as u64;

                let seq_sum = (0..cur_levels).fold(0, |acc, j| {
                    let j = j + i as u64;
                    acc + (2u64.pow(j as u32) * (depth - j))
                });

                let offset = i as u64;

                let sum_of_all_seq_numbers: u64 = jobs
                    .iter()
                    .map(|job| match job {
                        Job::Base(base::Record { seq_no, .. }) => seq_no.0 - offset,
                        Job::Merge(merge::Record { seq_no, .. }) => seq_no.0 - offset,
                    })
                    .sum();

                dbg!(sum_of_all_seq_numbers, seq_sum);
                assert_eq!(sum_of_all_seq_numbers, seq_sum);
            }
        };

        let mut state = ParallelScan::<i64, i64>::empty(max_base_jobs, 0);
        let mut counter = 0;

        for _ in 0..50 {
            let jobs = flatten(state.jobs_for_next_update());
            let jobs_done: Vec<_> = jobs.iter().map(job_done).collect();
            let data: Vec<i64> = (0..max_base_jobs as i64).collect();

            let (res_opt, s) = test_update(&state, data, jobs_done);

            state = s;

            if res_opt.is_some() {
                if counter > p {
                    verify_sequence_number(&state);
                } else {
                    counter += 1;
                }
            };
        }

        assert_eq!(
            hash(&state),
            "931a0dc0a488289000c195ae361138cc713deddc179b5d22bfa6344508d0cfb5"
        );
    }

    fn step_on_free_space<F, FAcc, B, M>(
        state: &mut ParallelScan<B, M>,
        w: &SyncSender<Option<(M, Vec<B>)>>,
        mut ds: Vec<B>,
        f: F,
        f_acc: FAcc,
    ) where
        F: Fn(&AvailableJob<B, M>) -> M,
        FAcc: Fn(Option<(M, Vec<B>)>, (M, Vec<B>)) -> Option<(M, Vec<B>)>,
        B: Debug + Clone + 'static,
        M: Debug + Clone + 'static,
    {
        loop {
            let data = take(&ds, state.max_base_jobs as usize);

            let jobs = flatten(state.work_for_next_update(data.len() as u64));
            let jobs_done: Vec<_> = jobs.iter().map(&f).collect();

            let old_tuple = state.acc.clone();

            let (res_opt, mut s) = test_update(state, data.to_vec(), jobs_done);

            let s = match res_opt {
                Some(x) => {
                    let tuple = if old_tuple.is_some() {
                        f_acc(old_tuple, x)
                    } else {
                        s.acc.clone()
                    };
                    s.acc = tuple;
                    s
                }
                None => s,
            };

            w.send(s.acc.clone()).unwrap();

            *state = s;

            let rem_ds = ds.get(state.max_base_jobs as usize..).unwrap_or(&[]);

            if rem_ds.is_empty() {
                return;
            } else {
                ds = rem_ds.to_vec();
            }
        }
    }

    fn do_steps<F, FAcc, B, M>(
        state: &mut ParallelScan<B, M>,
        recv: &Receiver<B>,
        f: F,
        f_acc: FAcc,
        w: SyncSender<Option<(M, Vec<B>)>>,
    ) where
        F: Fn(&AvailableJob<B, M>) -> M,
        FAcc: Fn(Option<(M, Vec<B>)>, (M, Vec<B>)) -> Option<(M, Vec<B>)>,
        B: Debug + Clone + 'static,
        M: Debug + Clone + 'static,
    {
        let data = recv.recv().unwrap();
        step_on_free_space(state, &w, vec![data], &f, &f_acc);
    }

    fn scan<F, FAcc, B, M>(
        s: &mut ParallelScan<B, M>,
        data: &Receiver<B>,
        f: F,
        f_acc: FAcc,
    ) -> Receiver<Option<(M, Vec<B>)>>
    where
        F: Fn(&AvailableJob<B, M>) -> M,
        FAcc: Fn(Option<(M, Vec<B>)>, (M, Vec<B>)) -> Option<(M, Vec<B>)>,
        B: Debug + Clone + 'static,
        M: Debug + Clone + 'static,
    {
        let (send, rec) = std::sync::mpsc::sync_channel::<Option<(M, Vec<B>)>>(1);
        do_steps(s, data, f, f_acc, send);
        rec
    }

    fn step_repeatedly<F, FAcc, B, M>(
        state: &mut ParallelScan<B, M>,
        data: &Receiver<B>,
        f: F,
        f_acc: FAcc,
    ) -> Receiver<Option<(M, Vec<B>)>>
    where
        F: Fn(&AvailableJob<B, M>) -> M,
        FAcc: Fn(Option<(M, Vec<B>)>, (M, Vec<B>)) -> Option<(M, Vec<B>)>,
        B: Debug + Clone + 'static,
        M: Debug + Clone + 'static,
    {
        let (send, rec) = std::sync::mpsc::sync_channel::<Option<(M, Vec<B>)>>(1);
        do_steps(state, data, f, f_acc, send);
        rec
    }

    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/parallel_scan/parallel_scan.ml#L1803
    #[test]
    fn scan_can_be_initialized_from_intermediate_state() {
        for _ in 0..10 {
            let mut state = gen(job_done, fun_merge_up);

            println!("state={:#?}", state);

            let do_one_next = Arc::new(AtomicBool::new(false));

            let do_one_next_clone = Arc::clone(&do_one_next);
            let (send, recv) = sync_channel(1);

            std::thread::spawn(move || loop {
                let v = if do_one_next_clone.load(Relaxed) {
                    do_one_next_clone.store(false, Relaxed);
                    1i64
                } else {
                    0i64
                };
                if send.send(v).is_err() {
                    return;
                }
            });

            let one_then_zeros = recv;

            let parallelism = state.max_base_jobs * ceil_log2(state.max_base_jobs);
            let old_acc = state.acc.as_ref().cloned().unwrap_or((0, vec![]));

            let fill_some_zero =
                |state: &mut ParallelScan<i64, i64>, v: i64, r: &Receiver<i64>| -> i64 {
                    (0..parallelism * parallelism).fold(v, |acc, _| {
                        let pipe = step_repeatedly(state, r, job_done, fun_merge_up);

                        match pipe.recv() {
                            Ok(Some((v, _))) => v,
                            Ok(None) => acc,
                            Err(_) => acc,
                        }
                    })
                };

            let v = fill_some_zero(&mut state, 0, &one_then_zeros);

            do_one_next.store(true, Relaxed);

            let acc = { state.acc.clone().unwrap() };

            assert_ne!(acc.0, old_acc.0);

            fill_some_zero(&mut state, v, &one_then_zeros);

            let acc_plus_one = { state.acc.unwrap() };
            assert_eq!(acc_plus_one.0, acc.0.wrapping_add(1));
        }
    }

    /// scan (+) over ints, map from string
    ///
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/parallel_scan/parallel_scan.ml#L1879
    #[test]
    fn scan_behaves_like_a_fold_long_term() {
        fn fun_merge_up(
            tuple: Option<(i64, Vec<String>)>,
            mut x: (i64, Vec<String>),
        ) -> Option<(i64, Vec<String>)> {
            let mut acc = tuple?;
            acc.1.append(&mut x.1);

            Some((acc.0.wrapping_add(x.0), acc.1))
        }

        fn job_done(job: &AvailableJob<String, i64>) -> i64 {
            match job {
                AvailableJob::Base(x) => x.parse().unwrap(),
                AvailableJob::Merge { left, right } => left.wrapping_add(*right),
            }
        }

        let (send, recv) = sync_channel(1);

        let depth = 7;
        let count: i64 = 1000;

        std::thread::spawn(move || {
            let mut count = count;
            let x = count;
            loop {
                let next = if count <= 0 {
                    "0".to_string()
                } else {
                    (x - count).to_string()
                };

                count -= 1;

                if send.send(next).is_err() {
                    return;
                }
            }
        });

        let mut s = ParallelScan::<String, i64>::empty(2u64.pow(depth as u32), 1);

        let after_3n = (0..4 * count).fold(0i64, |acc, _| {
            let result = scan(&mut s, &recv, job_done, fun_merge_up);
            match result.recv() {
                Ok(Some((v, _))) => v,
                Ok(None) => acc,
                Err(_) => acc,
            }
        });

        let expected = (0..count).fold(0i64, |a, b| a.wrapping_add(b));

        assert_eq!(after_3n, expected);
    }

    /// scan performs operation in correct order with \
    /// non-commutative semigroup
    ///
    /// https://github.com/MinaProtocol/mina/blob/2ee6e004ba8c6a0541056076aab22ea162f7eb3a/src/lib/parallel_scan/parallel_scan.ml#L1917
    #[test]
    fn scan_concat_over_strings() {
        fn fun_merge_up(
            tuple: Option<(String, Vec<String>)>,
            mut x: (String, Vec<String>),
        ) -> Option<(String, Vec<String>)> {
            let mut acc = tuple?;
            acc.1.append(&mut x.1);

            Some((format!("{}{}", acc.0, x.0), acc.1))
        }

        fn job_done(job: &AvailableJob<String, String>) -> String {
            match job {
                AvailableJob::Base(x) => x.clone(),
                AvailableJob::Merge { left, right } => {
                    format!("{}{}", left, right)
                }
            }
        }

        let (send, recv) = sync_channel(1);

        let depth = 7;
        let count: i64 = 100;

        std::thread::spawn(move || {
            let mut count = count;
            let x = count;
            loop {
                let next = if count <= 0 {
                    "".to_string()
                } else {
                    let n = (x - count).to_string();
                    format!("{},", n)
                };

                count -= 1;

                if send.send(next).is_err() {
                    return;
                }
            }
        });

        let mut s = ParallelScan::<String, String>::empty(2u64.pow(depth as u32), 1);

        let after_3n = (0..42 * count).fold(String::new(), |acc, _| {
            let result = scan(&mut s, &recv, job_done, fun_merge_up);
            match result.recv() {
                Ok(Some((v, _))) => v,
                Ok(None) => acc,
                Err(_) => acc,
            }
        });

        let expected = (0..count)
            .map(|i| format!("{},", i))
            .fold(String::new(), |a, b| format!("{}{}", a, b));

        assert_eq!(after_3n, expected);
    }
}
