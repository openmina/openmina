import { ScanStateState } from '@snarks/scan-state/scan-state.state';
import {
  SCAN_STATE_CLOSE,
  SCAN_STATE_GET_BLOCK_SUCCESS,
  SCAN_STATE_HIGHLIGHT_SNARK_POOL,
  SCAN_STATE_PAUSE,
  SCAN_STATE_SET_ACTIVE_JOB_ID,
  SCAN_STATE_SET_ACTIVE_LEAF,
  SCAN_STATE_SIDEBAR_RESIZED,
  SCAN_STATE_START,
  SCAN_STATE_TOGGLE_SIDE_PANEL,
  SCAN_STATE_TOGGLE_TREE_VIEW,
  ScanStateActions
} from '@snarks/scan-state/scan-state.actions';
import { isDesktop } from '@openmina/shared';

const initialState: ScanStateState = {
  block: undefined,
  activeJobId: undefined,
  activeLeaf: undefined,
  openSidePanel: isDesktop(),
  sideBarResized: 0,
  stream: true,
  treeView: JSON.parse(localStorage.getItem('scan_state_tree_view')) || false,
  highlightSnarkPool: true,
};

export function reducer(state: ScanStateState = initialState, action: ScanStateActions): ScanStateState {
  switch (action.type) {

    case SCAN_STATE_GET_BLOCK_SUCCESS: {
      return {
        ...state,
        block: action.payload,
        activeLeaf: state.activeJobId
          ? {
            ...action.payload.trees.reduce((acc, t) => [...acc, ...t.leafs.filter(l => l.bundle_job_id === state.activeJobId)], [])[0],
            scrolling: !state.activeLeaf,
          }
          : undefined,
      };
    }

    case SCAN_STATE_SET_ACTIVE_JOB_ID: {
      return {
        ...state,
        activeJobId: action.payload,
        stream: false,
      };
    }

    case SCAN_STATE_TOGGLE_SIDE_PANEL: {
      return {
        ...state,
        openSidePanel: !state.openSidePanel,
        activeJobId: undefined,
        activeLeaf: undefined,
      };
    }

    case SCAN_STATE_SET_ACTIVE_LEAF: {
      return {
        ...state,
        openSidePanel: action.payload ? true : state.openSidePanel,
        activeLeaf: action.payload,
        activeJobId: !action.payload ? undefined : action.payload.bundle_job_id,
        stream: action.payload ? false : state.stream,
      };
    }

    case SCAN_STATE_SIDEBAR_RESIZED: {
      return {
        ...state,
        sideBarResized: state.sideBarResized + 1,
      };
    }

    case SCAN_STATE_START: {
      return {
        ...state,
        stream: true,
      };
    }

    case SCAN_STATE_PAUSE: {
      return {
        ...state,
        stream: false,
      };
    }

    case SCAN_STATE_TOGGLE_TREE_VIEW: {
      localStorage.setItem('scan_state_tree_view', JSON.stringify(!state.treeView));

      return {
        ...state,
        treeView: !state.treeView,
      };
    }

    case SCAN_STATE_HIGHLIGHT_SNARK_POOL: {
      return {
        ...state,
        highlightSnarkPool: !state.highlightSnarkPool,
      };
    }

    case SCAN_STATE_CLOSE:
      return initialState;

    default:
      return state;
  }
}
