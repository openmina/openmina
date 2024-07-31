import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../support/commands';
import { DashboardState } from '@dashboard/dashboard.state';

const condition = (state: DashboardState): boolean => state && state.peers?.length > 1;
const getDashboard = (store: Store<MinaState>): DashboardState => stateSliceAsPromise<DashboardState>(store, condition, 'dashboard');

describe('DASHBOARD LEDGERS', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/dashboard');
  });

  it('should show ledgers progress', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          cy.get('mina-dashboard-ledger > div:first-child > div:first-child > div.f-400.tertiary')
            .then((span: any) => {
              const ledgers = state.nodes[0].ledgers;
              if (!ledgers.stakingEpoch.snarked.fetchHashesStart) {
                return;
              }
              let progress;
              if (ledgers.rootStaged.state === 'success') {
                progress = calculateProgressTime(ledgers.rootStaged.staged.reconstructEnd, 'finished');
              } else {
                progress = calculateProgressTime(ledgers.stakingEpoch.snarked.fetchHashesStart, 'started');
              }
              expect(span.text()).equals(progress);
            });
        }
      });
  });

  it('should show staking ledger percentage', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          cy.get('mina-dashboard-ledger > div:first-child > div.h-minus-xl > .group:nth-child(1) div.primary')
            .then((el: any) => {
              let stakingProgress;
              stakingProgress = state.rpcStats.stakingLedger?.fetched / state.rpcStats.stakingLedger?.estimation * 100 || 0;
              stakingProgress = Math.round(stakingProgress);

              if (state.nodes[0].ledgers.stakingEpoch.state === 'success') {
                stakingProgress = 100;
              }

              expect(el.text().trim()).equals(stakingProgress + '%');
            });
        }
      });
  });

  it('should show next epoch ledger percentage', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          cy.get('mina-dashboard-ledger > div:first-child > div.h-minus-xl > .group:nth-child(2) div.primary')
            .then((el: any) => {
              let progress;
              progress = state.rpcStats.nextLedger?.fetched / state.rpcStats.nextLedger?.estimation * 100 || 0;
              progress = Math.round(progress);

              if (state.nodes[0].ledgers.nextEpoch.state === 'success') {
                progress = 100;
              }

              expect(el.text().trim()).equals(progress + '%');
            });
        }
      });
  });

  it('should show snarked ledger percentage', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          cy.get('mina-dashboard-ledger > div:first-child > div.h-minus-xl > .group:nth-child(3) div.primary')
            .then((el: any) => {
              let progress;
              progress = state.rpcStats.rootLedger?.fetched / state.rpcStats.rootLedger?.estimation * 100 || 0;
              progress = Math.round(progress);

              if (state.nodes[0].ledgers.rootSnarked.state === 'success') {
                progress = 100;
              }

              expect(el.text().trim()).equals(progress + '%');
            });
        }
      });
  });

  it('should show staked ledger percentage', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          cy.get('mina-dashboard-ledger > div:first-child > div.h-minus-xl > .group:nth-child(4) div.primary')
            .then((el: any) => {
              let progress;
              progress = state.nodes[0].ledgers.rootStaged.staged.fetchPartsEnd ? 50 : 0;
              progress = Math.round(progress);

              if (state.nodes[0].ledgers.rootStaged.state === 'success') {
                progress = 100;
              }

              expect(el.text().trim()).equals(progress + '%');
            });
        }
      });
  });

});

function calculateProgressTime(timestamp: number, action: string): string {
  timestamp = Math.ceil(timestamp / 1000000);
  const millisecondsAgo = Date.now() - timestamp;
  const minutesAgo = Math.floor(millisecondsAgo / 1000 / 60);
  const hoursAgo = Math.floor(minutesAgo / 60);
  const daysAgo = Math.floor(hoursAgo / 24);

  if (daysAgo > 0) {
    return `${action} ${daysAgo}d ago`;
  } else if (hoursAgo > 0) {
    return `${action} ${hoursAgo}h ago`;
  } else if (minutesAgo > 0) {
    return `${action} ${minutesAgo}m ago`;
  } else {
    return `${action} <1m ago`;
  }
}
