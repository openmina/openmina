export interface NetworkConnection {
  connectionId: number;
  addr: string;
  pid: number;
  fd: number;
  incoming: string;
  date: string;
  timestamp: number;
  alias: string;
  stats_in: any;
  stats_out: any;
}
