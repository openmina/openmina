import { MinaState } from '@app/app.setup';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { AppNodeDetails } from '@shared/types/app/app-node-details.type';
import { AppEnvBuild } from '@shared/types/app/app-env-build.type';

export interface AppState {
  menu: AppMenu;
  nodes: MinaNode[];
  activeNode: MinaNode;
  activeNodeDetails: AppNodeDetails;
  envBuild: AppEnvBuild | undefined;
}

const select = <T>(selector: (state: AppState) => T): MemoizedSelector<MinaState, T> => createSelector(
  (state: MinaState): AppState => state.app,
  selector,
);

const menu = select(state => state.menu);
const nodes = select(state => state.nodes);
const activeNode = select(state => state.activeNode);
const activeNodeDetails = select(state => state.activeNodeDetails);
const envBuild = select(state => state.envBuild);

export const AppSelectors = {
  menu,
  nodes,
  activeNode,
  activeNodeDetails,
  envBuild,
};
