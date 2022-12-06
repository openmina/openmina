use std::rc::Rc;

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

/// A single tree with number of leaves = max_base_jobs = 2**transaction_capacity_log_2
#[derive(Debug)]
enum Tree<B, M> {
    Leaf(B),
    Node {
        depth: u64,
        value: M,
        sub_tree: Rc<Tree<(base::Base, base::Base), (merge::Merge, merge::Merge)>>,
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
