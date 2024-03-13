export interface NetworkNodeDHT {
  peerId: string;
  key: string;
  addressesLength: number;
  addrs: string[];
  hexDistance: string;
  binaryDistance: string;
  xorDistance: string;
  bucketIndex: number;
  bucketMaxHex: string;
}
