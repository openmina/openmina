export interface AppNodeDetails {
  status: AppNodeStatus;
  blockHeight: number;
  blockTime: number;

  peers: number;
  download: number;
  upload: number;

  transactions: number;
  snarks: number;
}

export enum AppNodeStatus {
  PENDING = 'Pending',
  BOOTSTRAP = 'Bootstrap',
  CATCHUP = 'Catchup',
  SYNCED = 'Synced',
  OFFLINE = 'Offline',
}
