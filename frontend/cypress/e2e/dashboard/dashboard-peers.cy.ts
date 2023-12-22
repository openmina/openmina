import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { checkSorting, Sort, stateSliceAsPromise } from '../../support/commands';
import { DashboardState } from '@dashboard/dashboard.state';

const condition = (state: DashboardState): boolean => state && state.peers?.length > 1;
const getDashboard = (store: Store<MinaState>): DashboardState => stateSliceAsPromise<DashboardState>(store, condition, 'dashboard');
const tableHead = (child: number) => `mina-dashboard-peers-table .head > span:nth-child(${child})`;

describe('DASHBOARD PEERS', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/dashboard');
  });

  it('should have correct title', () => {
    cy.wait(2000)
      .window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          cy.get('mina-toolbar span')
            .then((span: any) => expect(span).contain('Dashboard'));
        }
      });
  });

  it('display peers in the table', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          expect(state.peers.length).above(1);
          cy.get('mina-dashboard-peers-table .mina-table')
            .get('.row')
            .should('have.length.above', 1);
        }
      });
  });

  it('by default, sort table by date', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          checkSorting(state.peers, 'timestamp', Sort.DSC);
        }
      });
  });

  it('sort by peer id', () => {
    cy.get(tableHead(1))
      .click()
      .get(tableHead(1))
      .click()
      .window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          checkSorting(state.peers, 'peerId', Sort.ASC);
        }
      });
  });

  it('sort by status', () => {
    cy.get(tableHead(2))
      .click()
      .get(tableHead(2))
      .click()
      .window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          checkSorting(state.peers, 'status', Sort.ASC);
        }
      });
  });

  it('sort by datetime reversed', () => {
    cy.get(tableHead(3))
      .click()
      .window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          checkSorting(state.peers, 'timestamp', Sort.DSC);
        }
      });
  });

  it('sort by height', () => {
    cy.get(tableHead(6))
      .click()
      .get(tableHead(6))
      .click()
      .window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          checkSorting(state.peers, 'height', Sort.ASC);
        }
      });
  });

  it('sort by best tip', () => {
    cy.get(tableHead(8))
      .click()
      .get(tableHead(8))
      .click()
      .window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          checkSorting(state.peers, 'bestTip', Sort.ASC);
        }
      });
  });

  it('display correct summary data', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          cy.get('mina-dashboard-peers mina-card:nth-child(1) .value')
            .then((el) =>  expect(el.text().trim()).equals(state.peersStats.connecting.toString()));
          cy.get('mina-dashboard-peers mina-card:nth-child(2) .value')
            .then((el) =>  expect(el.text().trim()).equals(state.peersStats.connected.toString()));
          cy.get('mina-dashboard-peers mina-card:nth-child(3) .value')
            .then((el) =>  expect(el.text().trim()).equals(state.peersStats.disconnected.toString()));
        }
      });
  });
});
