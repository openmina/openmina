import { NodesOverviewState } from '@nodes/overview/nodes-overview.state';
import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { checkSorting, Sort, stateSliceAsPromise } from '../../../support/commands';
import { AppState } from '@app/app.state';

const condition = (state: NodesOverviewState) => state && state.nodes.length > 1;
const getNodesOverview = (store: Store<MinaState>) => stateSliceAsPromise<NodesOverviewState>(store, condition, 'nodes', 'overview');
const nodesCondition = (state: AppState) => state && state.nodes.length > 0;
const getNodes = (store: Store<MinaState>) => stateSliceAsPromise<AppState>(store, nodesCondition, 'app');

describe('NODES OVERVIEW TABLE', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/nodes/overview');
  });

  it('display overview title', () => {
    cy.wait(2000)
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          cy.get('mina-toolbar span')
            .then((span: any) => expect(span).contain('Nodes'));
        }
      });
  });

  it('display nodes in the table', () => {
    cy.window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          expect(state.nodes.length).above(1);
          cy.get('mina-nodes-overview .mina-table')
            .get('.row')
            .should('have.length.above', 1);
        }
      });
  });

  it('by default, sort table by status', () => {
    cy.window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          checkSorting(state.nodes, 'kind', Sort.DSC);
        }
      });
  });

  it('have expected length of nodes', () => {
    cy.window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          cy.window()
            .its('store')
            .then(getNodes)
            .then((state: AppState | void) => {
              if (state && state.nodes.length > 0) {
                const eachNodeHaveOneValue = state.nodes.every(n => state.nodes.filter(n1 => n1.url === n.url).length === 1);
                if (eachNodeHaveOneValue) {
                  expect(state.nodes.length).to.eq(state.nodes.length);
                } else {
                  expect(state.nodes.length).to.be.at.least(state.nodes.length);
                }
              }
            });
        }
      });
  });

  it('sort by name', () => {
    cy.get('mina-nodes-overview-table .head > span:nth-child(2)')
      .click()
      .get('mina-nodes-overview-table .head > span:nth-child(2)')
      .click()
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          checkSorting(state.nodes, 'name', Sort.ASC);
        }
      });
  });

  it('sort by status', () => {
    cy.get('mina-nodes-overview-table .head > span:nth-child(1)')
      .click()
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          checkSorting(state.nodes, 'kind', Sort.ASC);
        }
      });
  });

  it('sort by hash reversed', () => {
    cy.get('mina-nodes-overview-table .head > span:nth-child(4)')
      .click()
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          checkSorting(state.nodes, 'bestTip', Sort.DSC);
        }
      });
  });

  it('sort by height', () => {
    cy.get('mina-nodes-overview-table .head > span:nth-child(3)')
      .click()
      .get('mina-nodes-overview-table .head > span:nth-child(3)')
      .click()
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          checkSorting(state.nodes, 'height', Sort.ASC);
        }
      });
  });

  it('sort by date', () => {
    cy.get('mina-nodes-overview-table .head > span:nth-child(5)')
      .click()
      .get('mina-nodes-overview-table .head > span:nth-child(5)')
      .click()
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.nodes.length > 1) {
          checkSorting(state.nodes, 'bestTipReceivedTimestamp', Sort.ASC);
        }
      });
  });

  it('open side panel', () => {
    cy.get('mina-nodes-overview-table .row:not(.head)')
      .first()
      .click()
      .wait(1000)
      .window()
      .its('store')
      .then(getNodesOverview)
      .then((state: NodesOverviewState | void) => {
        if (state && state.activeNode) {
          expect(state.activeNode.name).to.eq(state.nodes[0].name);
          expect(state.activeNode.bestTip).to.eq(state.nodes[0].bestTip);
        }
      })
      .get('mina-nodes-overview-side-panel')
      .should('be.visible');
  });

});
