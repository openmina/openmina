export interface NetworkNodeDhtPeer {
  peerId: string;
  key: string;
  addressesLength: number;
  addrs: string[];
  libp2p: string;
  hexDistance: string;
  binaryDistance: string;
  xorDistance: string;
  bucketIndex: number;
  bucketMaxHex: string;
}
