import { AppActions } from '@app/app.actions';
import { AppState } from '@app/app.state';
import { MinaNode } from '@shared/types/core/environment/mina-env.type';
import { createReducer, on } from '@ngrx/store';
import { AppNodeStatus } from '@shared/types/app/app-node-details.type';

const initialState: AppState = {
  menu: {
    collapsed: JSON.parse(localStorage.getItem('menu_collapsed')) || false,
    isMobile: false,
    open: true,
  },
  nodes: [],
  activeNode: undefined,
  activeNodeDetails: {
    status: AppNodeStatus.PENDING,
    blockHeight: null,
    blockTime: null,
    peers: 0,
    download: 0,
    upload: 0,
    transactions: 0,
    snarks: 0,
  },
};

export const appReducer = createReducer(
  initialState,
  on(AppActions.init, (state) => ({ ...state })),
  on(AppActions.initSuccess, (state, { activeNode, nodes }) => ({ ...state, activeNode, nodes })),
  on(AppActions.changeActiveNode, (state, { node }) => ({ ...state, activeNode: node })),
  on(AppActions.getNodeDetailsSuccess, (state, { details }) => ({ ...state, activeNodeDetails: details })),
  on(AppActions.changeMenuCollapsing, (state, { isCollapsing }) => {
    localStorage.setItem('menu_collapsed', JSON.stringify(isCollapsing));
    return { ...state, menu: { ...state.menu, collapsed: isCollapsing } };
  }),
  on(AppActions.toggleMobile, (state, { isMobile }) => ({
    ...state,
    menu: { ...state.menu, isMobile, open: !isMobile },
  })),
  on(AppActions.toggleMenuOpening, (state) => ({ ...state, menu: { ...state.menu, open: !state.menu.open } })),
  on(AppActions.addNode, (state, { node }) => {
    const customNodes = JSON.parse(localStorage.getItem('custom_nodes') ?? '[]');
    localStorage.setItem('custom_nodes', JSON.stringify([node, ...customNodes]));
    return { ...state, nodes: [node, ...state.nodes] };
  }),
  on(AppActions.deleteNode, (state, { node }) => {
    const customNodes = JSON.parse(localStorage.getItem('custom_nodes') ?? '[]');
    localStorage.setItem('custom_nodes', JSON.stringify(customNodes.filter((n: MinaNode) => n.name !== node.name)));
    const nodes = state.nodes.filter(n => n.name !== node.name);
    return { ...state, nodes, activeNode: state.activeNode?.name === node.name ? nodes[0] : state.activeNode };
  }),
);
