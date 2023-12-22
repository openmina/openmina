import { NodesOverviewState } from '@nodes/overview/nodes-overview.state';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';

const condition = (state: NodesOverviewState) => state && state.nodes?.length > 0 && state.nodes.some(n => n.kind === 'Synced');
const getNodesOverview = (store: Store<MinaState>) => stateSliceAsPromise<NodesOverviewState>(store, condition, 'nodes', 'overview');

describe('NODES OVERVIEW SIDE PANEL', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/nodes/overview');
  });

  it('side panel block summary are correct', () => {
    cy.window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState) => {
        if (condition(state)) {
          cy.get('mina-nodes-overview-table .row:not(.head)')
            .first()
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(getNodesOverview)
            .then((state: NodesOverviewState) => {
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
            .wait(1000)
            .get('mina-nodes-overview-side-panel .mina-icon')
            .should('be.visible');
        }
      });
  });

  it('side panel transition frontier is correct', () => {
    cy.window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState) => {
        if (condition(state)) {
          cy.get('mina-nodes-overview-table .row:not(.head)')
            .first()
            .click()
            .wait(1000)
            .window()
            .its('store')
            .then(getNodesOverview)
            .then((state: NodesOverviewState) => {
              if (state && state.activeNode) {
                cy.get('mina-nodes-overview-side-panel .squares > div')
                  .should('have.length', 291)
                cy.get('mina-nodes-overview-side-panel .squares > div.Applied')
                  .should('have.length', state.activeNode.appliedBlocks)
                cy.get('mina-nodes-overview-side-panel .squares > div.Applying')
                  .should('have.length', state.activeNode.applyingBlocks)
                cy.get('mina-nodes-overview-side-panel .squares > div.Fetched')
                  .should('have.length', state.activeNode.fetchedBlocks)
                cy.get('mina-nodes-overview-side-panel .squares > div.Fetching')
                  .should('have.length', state.activeNode.fetchingBlocks)
                cy.get('mina-nodes-overview-side-panel .squares > div.Missing')
                  .should('have.length', state.activeNode.missingBlocks)
              }
            })
            .wait(1000)
            .get('mina-nodes-overview-side-panel .mina-icon')
            .should('be.visible');
        }
      });
  });

  it('close side panel', () => {
    cy.get('mina-nodes-overview-table .row:not(.head)')
      .first()
      .click()
      .wait(1000)
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState) => {
        if (state && state.activeNode) {
          expect(state.activeNode.name).to.eq(state.nodes[0].name);
          expect(state.activeNode.bestTip).to.eq(state.nodes[0].bestTip);
        }
      })
      .get('mina-nodes-overview-side-panel .mina-icon')
      .should('be.visible')
      .get('mina-nodes-overview-side-panel > div .mina-icon.pointer')
      .click()
      .wait(1000)
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState) => expect(state.activeNode).to.be.undefined)
      .wait(1000)
      .get('mina-nodes-overview-side-panel .mina-icon')
      .should('not.be.visible');
  });

});
