export interface WorkPoolCommitment {
  sender: string;
  received_t: number;
  date: string;
  commitment: {
    timestamp: number;
    snarker: string;
    fee: string;
    job_id: string;
  }
}
