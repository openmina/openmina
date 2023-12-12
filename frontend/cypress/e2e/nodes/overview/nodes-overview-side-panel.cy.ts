import { NodesOverviewState } from '@nodes/overview/nodes-overview.state';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { AppState } from '@app/app.state';

const condition = (state: NodesOverviewState | void) => state && state.nodes.length > 1;
const getNodesOverview = (store: Store<MinaState>) => stateSliceAsPromise<NodesOverviewState>(store, condition, 'nodes', 'overview');
const nodesCondition = (state: AppState) => state && state.nodes.length > 0;
const getNodes = (store: Store<MinaState>) => stateSliceAsPromise<AppState>(store, nodesCondition, 'app');

describe('NODES OVERVIEW SIDE PANEL', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/nodes/overview');
  });

  it('open side panel', () => {
    cy.window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (condition(state)) {
          cy.get('mina-nodes-overview-table .row:not(.head)')
            .first()
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(getNodesOverview)
            .then((state: NodesOverviewState | void) => {
              if (state && state.activeNode) {
                cy.get('mina-nodes-overview-side-panel .h-minus-lg .fx-row-vert-cent:nth-child(2)')
                  .then((el) => expect(el.text().trim()).equals(`Missing  ${state.activeNode.missingBlocks}`));
                cy.get('mina-nodes-overview-side-panel .h-minus-lg .fx-row-vert-cent:nth-child(3)')
                  .then((el) => expect(el.text().trim()).equals(`Fetching  ${state.activeNode.fetchingBlocks}`));
                cy.get('mina-nodes-overview-side-panel .h-minus-lg .fx-row-vert-cent:nth-child(4)')
                  .then((el) => expect(el.text().trim()).equals(`Fetched  ${state.activeNode.fetchedBlocks}`));
                cy.get('mina-nodes-overview-side-panel .h-minus-lg .fx-row-vert-cent:nth-child(5)')
                  .then((el) => expect(el.text().trim()).equals(`Applying  ${state.activeNode.applyingBlocks}`));
                cy.get('mina-nodes-overview-side-panel .h-minus-lg .fx-row-vert-cent:nth-child(6)')
                  .then((el) => expect(el.text().trim()).equals(`Applied  ${state.activeNode.appliedBlocks}`));
              }
            })
            .get('mina-nodes-overview-side-panel')
            .should('be.visible');
        }
      });
  });

});
