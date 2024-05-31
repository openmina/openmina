import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { createType } from '@shared/constants/store-functions';
import { createAction, props } from '@ngrx/store';
import { AppNodeDetails } from '@shared/types/app/app-node-details.type';

export const APP_KEY = 'app';
export const APP_PREFIX = 'App';

const type = <T extends string>(type: T) => createType(APP_PREFIX, null, type);

const init = createAction(type('Init'));
const initSuccess = createAction(type('Init Success'), props<{ activeNode: MinaNode, nodes: MinaNode[] }>());

const changeActiveNode = createAction(type('Change Active Node'), props<{ node: MinaNode }>());
const getNodeDetails = createAction(type('Get Node Details'));
const getNodeDetailsSuccess = createAction(type('Get Node Details Success'), props<{ details: AppNodeDetails }>());
const deleteNode = createAction(type('Delete Node'), props<{ node: MinaNode }>());
const addNode = createAction(type('Add Node'), props<{ node: MinaNode }>());

const changeMenuCollapsing = createAction(type('Change Menu Collapsing'), props<{ isCollapsing: boolean }>());
const changeSubMenus = createAction(type('Change Sub Menus'), props<{ subMenus: string[] }>());
const toggleMobile = createAction(type('Toggle Mobile'), props<{ isMobile: boolean }>());
const toggleMenuOpening = createAction(type('Toggle Menu Opening'));

export const AppActions = {
  init,
  initSuccess,
  changeActiveNode,
  getNodeDetails,
  getNodeDetailsSuccess,
  deleteNode,
  addNode,
  changeMenuCollapsing,
  changeSubMenus,
  toggleMobile,
  toggleMenuOpening,
};
