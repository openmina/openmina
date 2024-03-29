export interface NetworkNodeDhtPeer {
  peerId: string;
  key: string;
  addressesLength: number;
  addrs: string[];
  libp2p: string;
  connection: NetworkNodeDhtPeerConnectionType;
  hexDistance: string;
  binaryDistance: string;
  xorDistance: string;
  bucketMaxHex: string;
  bucketIndex: number;
  id?: number;
}

export enum NetworkNodeDhtPeerConnectionType {
  NotConnected = 'Not Connected',
  Connected = 'Connected',
  CanConnect = 'Can Connect',
  CannotConnect = 'Cannot Connect',
}
