import { MinaState } from '@app/app.setup';
import { AppMenu } from '@shared/types/app/app-menu.type';
import { createSelector, MemoizedSelector } from '@ngrx/store';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';

export interface AppState {
  subMenus: string[];
  menu: AppMenu;
  nodes: MinaNode[];
  activeNode: MinaNode;
}

const select = <T>(selector: (state: AppState) => T): MinaSelector<T> => createSelector(
  selectAppState,
  selector,
);

type MinaSelector<T> = MemoizedSelector<MinaState, T>;

export const selectAppState = (state: MinaState): AppState => state.app;
export const selectAppMenu = select((state: AppState): AppMenu => state.menu);
export const selectNodes: MinaSelector<MinaNode[]> = select(state => state.nodes);
export const selectActiveNode: MinaSelector<MinaNode> = select(state => state.activeNode);
