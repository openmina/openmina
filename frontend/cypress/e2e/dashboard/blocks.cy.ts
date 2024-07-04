import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../support/commands';
import { DashboardState } from '@dashboard/dashboard.state';
import { DashboardPeer } from '@shared/types/dashboard/dashboard.peer';

const condition = (state: DashboardState): boolean => state && state.peers?.length > 1;
const getDashboard = (store: Store<MinaState>): DashboardState => stateSliceAsPromise<DashboardState>(store, condition, 'dashboard');
const PENDING = 'Pending';
const SYNCED = 'Synced';

describe('DASHBOARD BLOCKS', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/dashboard');
  });

  it('should show fetched blocks', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          const nodes = state.nodes;
          let blocks = nodes[0].blocks;

          blocks = blocks.slice(1);

          const fetched = blocks.filter(b => ![NodesOverviewNodeBlockStatus.MISSING, NodesOverviewNodeBlockStatus.FETCHING].includes(b.status)).length;
          const fetchedPercentage = Math.round(fetched * 100 / 291) + '%';
          const expectedColorVar = fetchedPercentage === '100%' ? '--success-primary' : '--base-primary';
          const expectedStyle = `color: var(${expectedColorVar});`;

          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(1) > div:nth-child(2)')
            .should('have.attr', 'style', expectedStyle)
            .then((span: any) => {
              expect(span.text()).equals(fetchedPercentage);
            });
          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(1) > div:nth-child(3)')
            .then((span: any) => {
              expect(span.text()).equals(fetched + '/290 blocks');
            });
        }
      });
  });

  it('should show applied blocks', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          const nodes = state.nodes;
          let blocks = nodes[0].blocks;

          blocks = blocks.slice(1);

          const applied = blocks.filter(b => ![NodesOverviewNodeBlockStatus.MISSING, NodesOverviewNodeBlockStatus.FETCHING].includes(b.status)).length;
          const appliedPercentage = Math.round(applied * 100 / 291);

          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(2) > div:nth-child(2)')
            .then((span: any) => {
              expect(span.text()).equals(appliedPercentage !== undefined ? appliedPercentage + '%' : '-');
            });
          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(2) > div:nth-child(3)')
            .then((span: any) => {
              expect(span.text()).equals(applied + '/290 blocks');
            });
        }
      });
  });

  it('should show root', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          const nodes = state.nodes;
          let blocks = nodes[0].blocks;
          let root: number;
          let rootText: string;
          if (blocks.length === 291) {
            root = blocks[blocks.length - 1].height;
            rootText = calculateProgressTime(blocks[blocks.length - 1].applyEnd);
          } else {
            root = null;
            rootText = PENDING;
          }

          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(3) > div:nth-child(2)')
            .then((span: any) => {
              expect(span.text()).equals(root.toString());
            });
          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(3) > div:nth-child(3)')
            .then((span: any) => {
              expect(span.text()).equals(rootText);
            });
        }
      });
  });

  it('should show target best tip', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          const nodes = state.nodes;
          let blocks = nodes[0].blocks;
          let bestTipBlock: number;
          let bestTipBlockSyncedText: string;

          if (blocks.length > 0) {
            bestTipBlock = blocks[0].height;
            bestTipBlockSyncedText = 'Fetched ' + calculateProgressTime(nodes[0].bestTipReceivedTimestamp * 1e6).slice(7);
          }
          if (blocks.length === 291) {
            if (blocks[0].status === NodesOverviewNodeBlockStatus.APPLIED) {
              bestTipBlockSyncedText = SYNCED + ' ' + bestTipBlockSyncedText.slice(7);
            }
          }

          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(4) > div:nth-child(2)')
            .then((span: any) => {
              expect(span.text()).equals(bestTipBlock.toString());
            });
          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(4) > div:nth-child(3)')
            .then((span: any) => {
              expect(span.text()).equals(bestTipBlockSyncedText);
            });
        }
      });
  });


  it('should show max observed', () => {
    cy.window()
      .its('store')
      .then(getDashboard)
      .then((state: DashboardState) => {
        if (condition(state)) {
          const peers = state.peers;
          const highestHeightPeer = peers.reduce(
            (acc: DashboardPeer, peer: DashboardPeer) => peer.height > acc.height ? peer : acc,
            { height: 0 } as DashboardPeer,
          );
          const targetBlock = highestHeightPeer.height;

          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(5) > div:nth-child(2)')
            .then((span: any) => {
              expect(span.text()).equals(targetBlock.toString() || '');
            });
          cy.get('mina-dashboard-blocks-sync div > mina-card:nth-child(5) > div:nth-child(3)')
            .then((span: any) => {
              expect(span.text()).equals(targetBlock ? 'Now' : 'Waiting peers');
            });
        }
      });
  });

});

enum NodesOverviewNodeBlockStatus {
  APPLIED = 'Applied',
  APPLYING = 'Applying',
  FETCHED = 'Fetched',
  FETCHING = 'Fetching',
  MISSING = 'Missing',
}

function calculateProgressTime(timestamp: number): string {
  if (!timestamp) {
    return 'Pending';
  }
  timestamp = Math.ceil(timestamp / 1e6);
  const millisecondsAgo = Date.now() - timestamp;
  const minutesAgo = Math.floor(millisecondsAgo / 1000 / 60);
  const hoursAgo = Math.floor(minutesAgo / 60);
  const daysAgo = Math.floor(hoursAgo / 24);

  if (daysAgo > 0) {
    return `${SYNCED} ${daysAgo}d ago`;
  } else if (hoursAgo > 0) {
    return `${SYNCED} ${hoursAgo}h ago`;
  } else if (minutesAgo > 0) {
    return `${SYNCED} ${minutesAgo}m ago`;
  } else {
    return `${SYNCED} <1m ago`;
  }
}
