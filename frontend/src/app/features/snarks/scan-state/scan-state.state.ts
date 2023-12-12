import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { ScanStateBlock } from '@shared/types/snarks/scan-state/scan-state-block.type';
import { selectSnarksScanStateState } from '@snarks/snarks.state';
import { ScanStateLeaf } from '@shared/types/snarks/scan-state/scan-state-leaf.type';

export interface ScanStateState {
  block: ScanStateBlock;
  activeJobId: string;
  activeLeaf: ScanStateLeaf;
  openSidePanel: boolean;
  sideBarResized: number;
  stream: boolean;
  treeView: boolean;
  highlightSnarkPool: boolean;
}

const select = <T>(selector: (state: ScanStateState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectSnarksScanStateState,
  selector,
);

export const selectScanStateBlock = select((state: ScanStateState): ScanStateBlock => state.block);
export const selectScanStateActiveJobId = select((state: ScanStateState): string => state.activeJobId);
export const selectScanStateOpenSidePanel = select((state: ScanStateState): boolean => state.openSidePanel);
export const selectScanStateActiveLeaf = select((state: ScanStateState): ScanStateLeaf => state.activeLeaf);
export const selectScanStateSideBarResized = select((state: ScanStateState): number => state.sideBarResized);
export const selectScanStateStream = select((state: ScanStateState): boolean => state.stream);
export const selectScanStateTreeView = select((state: ScanStateState): boolean => state.treeView);
export const selectScanStateHighlightSnarkPool = select((state: ScanStateState): boolean => state.highlightSnarkPool);
