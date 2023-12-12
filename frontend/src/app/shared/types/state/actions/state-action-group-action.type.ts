import { StateActionColumn } from '@shared/types/state/actions/state-action-column.type';

export interface StateActionGroupAction {
  display: boolean;
  title: string;
  fullTitle: string;
  totalTime: number;
  meanTime: number;
  totalCount: number;
  columns: StateActionColumn[];
}
