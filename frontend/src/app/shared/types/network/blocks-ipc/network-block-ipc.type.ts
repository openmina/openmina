export interface NetworkBlockIpc {
  date: string;
  timestamp: number;
  nodeAddress: string;
  type: string;
  peerId: string;
  peerAddress: string;
  msgType: string;
  height: number;
  hash: string;
  blockLatency: number;
  realBlockLatency: number;
}
