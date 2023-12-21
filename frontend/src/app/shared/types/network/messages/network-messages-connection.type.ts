export interface NetworkMessageConnection {
  address: string;
  pid: number;
  fd: number;
  incoming: boolean;
  timestamp: string;
}
