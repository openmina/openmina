import { FeatureAction } from '@openmina/shared';
import { FeaturesConfig, MinaNode } from '@shared/types/core/environment/mina-env.type';

enum AppActionTypes {
  APP_INIT = 'APP_INIT',
  APP_INIT_SUCCESS = 'APP_INIT_SUCCESS',
  APP_CHANGE_ACTIVE_NODE = 'APP_CHANGE_ACTIVE_NODE',
  APP_DELETE_NODE = 'APP_DELETE_NODE',
  APP_ADD_NODE = 'APP_ADD_NODE',
  APP_CHANGE_MENU_COLLAPSING = 'APP_CHANGE_MENU_COLLAPSING',
  APP_CHANGE_SUB_MENUS = 'APP_CHANGE_SUB_MENUS',
  APP_TOGGLE_MOBILE = 'APP_TOGGLE_MOBILE',
  APP_TOGGLE_MENU_OPENING = 'APP_TOGGLE_MENU_OPENING',
}

export const APP_INIT = AppActionTypes.APP_INIT;
export const APP_INIT_SUCCESS = AppActionTypes.APP_INIT_SUCCESS;
export const APP_CHANGE_ACTIVE_NODE = AppActionTypes.APP_CHANGE_ACTIVE_NODE;
export const APP_DELETE_NODE = AppActionTypes.APP_DELETE_NODE;
export const APP_ADD_NODE = AppActionTypes.APP_ADD_NODE;
export const APP_CHANGE_MENU_COLLAPSING = AppActionTypes.APP_CHANGE_MENU_COLLAPSING;
export const APP_CHANGE_SUB_MENUS = AppActionTypes.APP_CHANGE_SUB_MENUS;
export const APP_TOGGLE_MOBILE = AppActionTypes.APP_TOGGLE_MOBILE;
export const APP_TOGGLE_MENU_OPENING = AppActionTypes.APP_TOGGLE_MENU_OPENING;

export interface AppAction extends FeatureAction<AppActionTypes> {
  readonly type: AppActionTypes;
}

export class AppInit implements AppAction {
  readonly type = APP_INIT;
}

export class AppInitSuccess implements AppAction {
  readonly type = APP_INIT_SUCCESS;

  constructor(public payload: { activeNode: MinaNode, nodes: MinaNode[] }) {}
}

export class AppChangeActiveNode implements AppAction {
  readonly type = APP_CHANGE_ACTIVE_NODE;

  constructor(public payload: MinaNode) { }
}

export class AppDeleteNode implements AppAction {
  readonly type = APP_DELETE_NODE;

  constructor(public payload: MinaNode) { }
}

export class AppAddNode implements AppAction {
  readonly type = APP_ADD_NODE;

  constructor(public payload: MinaNode) { }
}

export class AppChangeMenuCollapsing implements AppAction {
  readonly type = APP_CHANGE_MENU_COLLAPSING;

  constructor(public payload: boolean) { }
}

export class AppChangeSubMenus implements AppAction {
  readonly type = APP_CHANGE_SUB_MENUS;

  constructor(public payload: string[]) {} // todo: maybe add complex routes?
}

export class AppToggleMobile implements AppAction {
  readonly type = APP_TOGGLE_MOBILE;

  constructor(public payload: { isMobile: boolean }) { }
}

export class AppToggleMenuOpening implements AppAction {
  readonly type = APP_TOGGLE_MENU_OPENING;
}

export type AppActions =
  | AppInit
  | AppInitSuccess
  | AppAddNode
  | AppChangeActiveNode
  | AppDeleteNode
  | AppChangeMenuCollapsing
  | AppChangeSubMenus
  | AppToggleMobile
  | AppToggleMenuOpening
  ;
