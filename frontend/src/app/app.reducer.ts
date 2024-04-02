import {
  APP_ADD_NODE,
  APP_CHANGE_ACTIVE_NODE,
  APP_CHANGE_MENU_COLLAPSING, APP_DELETE_NODE,
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
  nodes: [],
  activeNode: undefined,
};

export function appReducer(state: AppState = initialState, action: any): AppState {
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
      const customNodes = localStorage.getItem('custom_nodes') ? JSON.parse(localStorage.getItem('custom_nodes')) : [];
      localStorage.setItem('custom_nodes', JSON.stringify([action.payload, ...customNodes]));
      return {
        ...state,
        nodes: [action.payload, ...state.nodes],
      };
    }

    case APP_DELETE_NODE: {
      const customNodes = localStorage.getItem('custom_nodes') ? JSON.parse(localStorage.getItem('custom_nodes')) : [];
      localStorage.setItem('custom_nodes', JSON.stringify(customNodes.filter((node: MinaNode) => node.name !== action.payload.name)));
      const nodes = state.nodes.filter(node => node.name !== action.payload.name);
      return {
        ...state,
        nodes,
        activeNode: state.activeNode?.name === action.payload.name ? nodes[0] : state.activeNode,
      };
    }

    default:
      return state;
  }
}
