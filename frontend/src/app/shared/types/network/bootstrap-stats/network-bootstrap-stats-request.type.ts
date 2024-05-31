export interface NetworkBootstrapStatsRequest {
  type: string;
  address: string;
  start: number;
  finish: number;
  durationInSecs: number;
  peerId: string;
  existingPeers: number;
  newPeers: number;
  error: string | undefined;
  typeErr: string;
  closestPeers: [PeerId, NetworkBootstrapPeerType][];
}

export type PeerId = string;

export enum NetworkBootstrapPeerType {
  NEW = 'New',
  EXISTING = 'Existing',
}
