use std::ops::ControlFlow;
use std::{fmt::Debug, rc::Rc};

use ControlFlow::{Break, Continue};

// type ControlFlow<T> = std::ops::ControlFlow<T, T>;

/// Sequence number for jobs in the scan state that corresponds to the order in
/// which they were added
#[derive(Copy, Clone, Debug)]
struct SequenceNumber(u64);

/// Each node on the tree is viewed as a job that needs to be completed. When a
/// job is completed, it creates a new "Todo" job and marks the old job as "Done"
#[derive(Clone, Debug)]
enum JobStatus {
    Todo,
    Done,
}

/// The number of new jobs- base and merge that can be added to this tree.
/// Each node has a weight associated to it and the
/// new jobs received are distributed across the tree based on this number.
#[derive(Clone, Debug)]
struct Weight {
    base: u64,
    merge: u64,
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

                let value = fun_merge(jobs.clone(), depth, value.clone())?;

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
                        value: value.0,
                        sub_tree,
                    },
                    value.1,
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
    // FunMerge: Fn(Data, u64, &M) -> Result<M, ()>,
}

impl Tree<base::Base, merge::Merge>
// where
//     B: Debug + 'static,
// M: Debug + 'static,
{
    fn update<WeightLens, WeightLensSet>(
        &self,
        completed_jobs: Vec<Job>,
        update_level: u64,
        sequence_no: SequenceNumber,
        weight_lens: WeightLens,
        weight_lens_set: WeightLensSet,
    ) -> Result<(Self, Option<base::Base>), ()>
    where
        WeightLens: Fn(&Weight) -> u64,
        WeightLensSet: Fn(&Weight, u64) -> Weight,
        // M: Clone,
    {
        let add_merges = |jobs: Vec<Job>, current_level: u64, merge_job: merge::Merge|
                     -> Result<(merge::Merge, Option<MergeJob>), ()>
            // |jobs: &[Job], current_level: u64, (weight, m): ((Weight, Weight), merge::Job)|
            //  -> Result<(((Weight, Weight), merge::Job), Option<MergeJob>), ()>
        {
            let weight = merge_job.weight;
            // let weight = (merge_job.weight_left, merge_job.weight_right);
            // let w1 = &merge_job.weight_left;
            // let w2 = &merge_job.weight_right;
            let m = merge_job.job;

            let (w1, w2) = (&weight.0, &weight.1);
            let (left, right) = (weight_lens(w1), weight_lens(w2));

            use merge::Job::{Empty, Full, Part};
            use Job::{Base, Merge};

            if current_level == update_level - 1 {
                // Create new jobs from the completed ones
                let (new_weight, new_m) = match (&jobs[..], m) {
                    ([], m) => (weight, m),
                    ([Merge(a), Merge(b)], Empty) => {
                        let w1 = weight_lens_set(w1, left - 1);
                        let w2 = weight_lens_set(w2, right - 1);

                        (
                            (w1, w2),
                            Full(merge::Record {
                                left: a.clone(),
                                right: b.clone(),
                                seq_no: sequence_no,
                                state: JobStatus::Todo,
                            }),
                        )
                    }
                    ([Merge(a)], Empty) => {
                        let w1 = weight_lens_set(w1, left - 1);
                        let w2 = weight_lens_set(w2, right);

                        ((w1, w2), Part(a.clone()))
                    }
                    ([Merge(b)], Part(a)) => {
                        let w1 = weight_lens_set(w1, left);
                        let w2 = weight_lens_set(w2, right - 1);

                        (
                            (w1, w2),
                            Full(merge::Record {
                                left: a.clone(),
                                right: b.clone(),
                                seq_no: sequence_no,
                                state: JobStatus::Todo,
                            }),
                        )
                    }
                    ([Base(_)], Empty) => {
                        // Depending on whether this is the first or second of the two base jobs

                        let weight = if left == 0 {
                            let w1 = weight_lens_set(w1, left);
                            let w2 = weight_lens_set(w2, right - 1);
                            (w1, w2)
                        } else {
                            let w1 = weight_lens_set(w1, left - 1);
                            let w2 = weight_lens_set(w2, right);
                            (w1, w2)
                        };

                        (weight, Empty)
                    }
                    ([Base(_), Base(_)], Empty) => {
                        let w1 = weight_lens_set(w1, left - 1);
                        let w2 = weight_lens_set(w2, right - 1);

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

                Ok((merge::Merge {
                    weight: new_weight,
                    job: new_m,
                }, None::<MergeJob>))

                // Ok(((new_weight, new_m), None::<MergeJob>))
            } else if current_level == update_level {
                // Mark completed jobs as Done

                match (jobs.as_slice(), m) {
                    (
                        [Merge(a)],
                        Full(
                            mut x @ merge::Record {
                                state: JobStatus::Todo,
                                ..
                            },
                        ),
                    ) => {
                        x.state = JobStatus::Done;
                        let new_job = Full(x);

                        let (scan_result, weight) = if current_level == 0 {
                            let w1 = weight_lens_set(w1, 0);
                            let w2 = weight_lens_set(w2, 0);

                            (Some(a.clone()), (w1, w2))
                        } else {
                            (None, weight)
                        };

                        Ok((merge::Merge {
                            weight,
                            job: new_job,
                        }, scan_result))
                        // Ok(((weight, new_job), scan_result))
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
                    // [] => Ok(((weight, m), None)),
                    _ => {
                        let jobs_length = jobs.len() as u64;
                        let jobs_sent_left = jobs_length.min(left);
                        let jobs_sent_right = (jobs_length - jobs_sent_left).min(right);

                        let w1 = weight_lens_set(w1, left - jobs_sent_left);
                        let w2 = weight_lens_set(w2, right - jobs_sent_right);
                        let weight = (w1, w2);

                        Ok((merge::Merge { weight, job: m }, None))
                        // Ok((((w1, w2), m), None))
                    }
                }
            } else {
                Ok((merge::Merge { weight, job: m }, None))
                // Ok(((weight, m), None))
            }
        };

        // let add_bases = |jobs: &[Job], (w, d): (Weight, base::Job)| {
        let add_bases = |jobs: Vec<Job>, base: base::Base| {
            use base::Job::{Empty, Full};
            use Job::{Base, Merge};

            let w = base.weight;
            let d = base.job;

            let weight = weight_lens(&w);
            match (jobs.as_slice(), d) {
                ([], d) => {
                    let base = base::Base { weight: w, job: d };
                    Ok::<_, ()>(base)
                }
                // ([], d) => (w, d),
                ([Base(d)], Empty) => {
                    let w = weight_lens_set(&w, weight - 1);

                    let base = base::Base {
                        weight: w,
                        job: Full(base::Record {
                            job: d.clone(),
                            seq_no: sequence_no,
                            state: JobStatus::Done,
                        }),
                    };

                    Ok(base)
                    // (
                    //     w,
                    //     Full(base::Record {
                    //         job: d.clone(),
                    //         seq_no: sequence_no,
                    //         state: JobStatus::Done,
                    //     }),
                    // )
                }
                ([Merge(_)], Full(mut b)) => {
                    b.state = JobStatus::Done;

                    let base = base::Base {
                        weight: w,
                        job: Full(b),
                    };

                    Ok(base)

                    // (w, Full(b))
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

        let jobs = completed_jobs;
        self.update_split(
            add_merges,
            add_bases,
            |m| {
                m.weight.clone() // TODO: Not sure if it's correct
            },
            jobs,
            update_level,
            |(w1, w2), a| {
                let l = weight_lens(&w1);
                let r = weight_lens(&w2);

                let take =
                    |v: &[Job], n: u64| v.iter().take(n as usize).cloned().collect::<Vec<Job>>();
                let drop =
                    |v: &[Job], n: u64| v.iter().skip(n as usize).cloned().collect::<Vec<Job>>();

                (take(a.as_slice(), l), take(&drop(a.as_slice(), l), r))
            },
        );

        // let jobs = completed_jobs in
        // update_split ~f_merge:add_merges ~f_base:add_bases tree ~weight_merge:fst
        //   ~jobs ~update_level ~jobs_split:(fun (w1, w2) a ->
        //     let l = weight_lens.get w1 in
        //     let r = weight_lens.get w2 in
        //     (List.take a l, List.take (List.drop a l) r) )

        todo!()
    }
}
