import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { selectNetworkBootstrapStatsState } from '@network/network.state';
import { TableSort } from '@openmina/shared';
import {
  NetworkBootstrapStatsRequest,
} from '@shared/types/network/bootstrap-stats/network-bootstrap-stats-request.type';

export interface NetworkBootstrapStatsState {
  boostrapStats: NetworkBootstrapStatsRequest[];
  activeBootstrapRequest: NetworkBootstrapStatsRequest;
  sort: TableSort<NetworkBootstrapStatsRequest>;
}

const select = <T>(selector: (state: NetworkBootstrapStatsState) => T): MemoizedSelector<MinaState, T> => createSelector(
  selectNetworkBootstrapStatsState,
  selector,
);

export const selectNetworkBootstrapStatsList = select((network: NetworkBootstrapStatsState): NetworkBootstrapStatsRequest[] => network.boostrapStats);
export const selectNetworkBootstrapStatsActiveBootstrapRequest = select((network: NetworkBootstrapStatsState): NetworkBootstrapStatsRequest => network.activeBootstrapRequest);
export const selectNetworkBootstrapStatsSort = select((state: NetworkBootstrapStatsState): TableSort<NetworkBootstrapStatsRequest> => state.sort);
