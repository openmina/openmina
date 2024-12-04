import { MinaNetwork } from '@shared/types/core/mina/mina.type';
import { BlockProductionWonSlotsStatus } from '@shared/types/block-production/won-slots/block-production-won-slots-slot.type';

export interface AppNodeDetails {
  status: AppNodeStatus;
  blockHeight: number;
  blockTime: number;

  peersConnected: number;
  peersDisconnected: number;
  peersConnecting: number;

  transactions: number;
  snarks: number;

  producingBlockAt: number;
  producingBlockGlobalSlot: number;
  producingBlockStatus: BlockProductionWonSlotsStatus;

  chainId?: string;
  network?: MinaNetwork;
}

export enum AppNodeStatus {
  PENDING = 'Pending',
  BOOTSTRAP = 'Bootstrap',
  CATCHUP = 'Catchup',
  SYNCED = 'Synced',
  OFFLINE = 'Offline',
}
