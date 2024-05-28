import {
  BlockProductionOverviewEpochDetails,
} from '@shared/types/block-production/overview/block-production-overview-epoch-details.type';
import {
  BlockProductionOverviewWindow,
} from '@shared/types/block-production/overview/block-production-overview-window.type';
import {
  BlockProductionOverviewSlot,
} from '@shared/types/block-production/overview/block-production-overview-slot.type';

export interface BlockProductionOverviewEpoch {
  epochNumber: number;
  windows: BlockProductionOverviewWindow[];
  finishedWindows: number;
  details?: BlockProductionOverviewEpochDetails;
  slots?: BlockProductionOverviewSlot[];
  isLastEpoch: boolean;
}
