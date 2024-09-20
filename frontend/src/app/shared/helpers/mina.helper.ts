import { MinaNetwork } from '@shared/types/core/mina/mina.type';

export function getNetwork(chainId: string): MinaNetwork {
  if (chainId === 'a7351abc7ddf2ea92d1b38cc8e636c271c1dfd2c081c637f62ebc2af34eb7cc1') {
    return MinaNetwork.MAINNET;
  } else if (chainId === '29936104443aaf264a7f0192ac64b1c7173198c1ed404c1bcff5e562e05eb7f6') {
    return MinaNetwork.DEVNET;
  }
  return MinaNetwork.UNKNOWN;
}
