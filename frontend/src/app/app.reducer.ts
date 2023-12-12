import {
  APP_ADD_NODE,
  APP_CHANGE_ACTIVE_NODE,
  APP_CHANGE_MENU_COLLAPSING,
  APP_CHANGE_SUB_MENUS,
  APP_INIT,
  APP_INIT_SUCCESS,
  APP_TOGGLE_MENU_OPENING,
  APP_TOGGLE_MOBILE,
} from '@app/app.actions';
import { AppState } from '@app/app.state';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';


const initialState: AppState = {
  menu: {
    collapsed: JSON.parse(localStorage.getItem('menu_collapsed')) || false,
    isMobile: false,
    open: true,
  },
  subMenus: [],
  nodes: [],
  activeNode: undefined,
};

export function reducer(state: AppState = initialState, action: any): AppState {
  switch (action.type) {

    case APP_INIT: {
      return {
        ...state,
      };
    }

    case APP_INIT_SUCCESS: {
      return {
        ...state,
        nodes: action.payload.nodes,
        activeNode: action.payload.activeNode,
      };
    }

    case APP_CHANGE_ACTIVE_NODE: {
      return {
        ...state,
        activeNode: action.payload,
      };
    }

    case APP_CHANGE_MENU_COLLAPSING: {
      localStorage.setItem('menu_collapsed', JSON.stringify(action.payload));
      return {
        ...state,
        menu: {
          ...state.menu,
          collapsed: action.payload,
        },
      };
    }

    case APP_CHANGE_SUB_MENUS: {
      return {
        ...state,
        subMenus: action.payload.filter(Boolean),
      };
    }

    case APP_TOGGLE_MOBILE: {
      return {
        ...state,
        menu: {
          ...state.menu,
          isMobile: action.payload.isMobile,
          open: !action.payload.isMobile,
        },
      };
    }

    case APP_TOGGLE_MENU_OPENING: {
      return {
        ...state,
        menu: {
          ...state.menu,
          open: !state.menu.open,
        },
      };
    }

    case APP_ADD_NODE: {
      const newNode: MinaNode = {
        features: {//todo: add this
          // overview: ['nodes', 'topology'],
          // explorer: ['blocks', 'transactions', 'snark-pool', 'scan-state', 'snark-traces'],
          // resources: ['system'],
          // network: ['messages', 'connections', 'blocks', 'blocks-ipc'],
          // tracing: ['overview', 'blocks'],
          // benchmarks: ['wallets'],
          // 'web-node': ['wallet', 'peers', 'logs', 'state'],
        },
        url: action.payload,
        name: action.payload.split('/')[action.payload.split('/').length - 1] || ('custom-node' + ++state.nodes.filter(n => n.name.includes('custom-node')).length),
      };
      return {
        ...state,
        nodes: [newNode, ...state.nodes],
      };
    }

    default:
      return state;
  }
}
