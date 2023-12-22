export interface ScanStateLeaf {
  status: ScanStateLeafStatus;
  job_id?: string;
  bundle_job_id?: string;
  job?: {
    kind: string;
  }
  seq_no?: number;
  commitment?: any;
  snark?: {
    snarker: string;
    fee: string;
    received_t?: number;
    sender?: string;
  };
  //frontend data
  jobIndex?: number;
  treeIndex?: number;
  scrolling?: boolean;
}

export enum ScanStateLeafStatus {
  Empty = 'Empty',
  Pending = 'Pending',
  Done = 'Done',
  Todo = 'Todo',
}
