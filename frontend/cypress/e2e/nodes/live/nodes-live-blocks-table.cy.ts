import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { NodesLiveState } from '@nodes/live/nodes-live.state';
import { lastItem } from '@openmina/shared';

const condition = (state: NodesLiveState) => state && state.nodes?.length > 0;
const getNodesLive = (store: Store<MinaState>) => stateSliceAsPromise<NodesLiveState>(store, condition, 'nodes', 'live');

describe('NODES LIVE BLOCKS TABLE', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/nodes/live');
  });

  it('display correct amount of blocks', () => {
    cy.window()
      .its('store')
      .then(getNodesLive)
      .then((state: NodesLiveState) => {
        if (condition(state)) {
          const node = state.activeNode;
          let blocks = (node?.blocks || []);
          if (blocks.length === 291) {
            blocks = blocks.slice(1);
          }
          if (blocks.length > 0) {
            blocks = blocks.slice(0, -1);
          }
          cy.get('mina-nodes-live-blocks-map .block')
            .should('have.length', blocks.length + 2);
          cy.get('mina-nodes-live-blocks-map .block.root-block')
            .should('have.length', 1);
          cy.get('mina-nodes-live-blocks-map .block.best-tip-block')
            .should('have.length', 1);
        }
      });
  });
});
