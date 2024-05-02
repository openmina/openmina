import { MinaState } from '@app/app.setup';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

export interface AppState {
  menu: AppMenu;
  nodes: MinaNode[];
  activeNode: MinaNode;
}

const select = <T>(selector: (state: AppState) => T): MinaSelector<T> => createSelector(
  (state: MinaState): AppState => state.app,
  selector,
);

type MinaSelector<T> = MemoizedSelector<MinaState, T>;

export const menu = select((state: AppState): AppMenu => state.menu);
export const nodes: MinaSelector<MinaNode[]> = select(state => state.nodes);
export const activeNode: MinaSelector<MinaNode> = select(state => state.activeNode);

export const AppSelectors = {
  menu,
  nodes,
  activeNode,
};
