import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';
import { SlotResponse } from '@block-production/overview/block-production-overview.service';

const condition = (state: BlockProductionOverviewState): boolean => state && state.epochs?.length > 0;
const getBPOverview = (store: Store<MinaState>): BlockProductionOverviewState => stateSliceAsPromise<BlockProductionOverviewState>(store, condition, 'blockProduction', 'overview');
const execute = (callback: () => void) => {
  cy.wait('@slotsRequest')
    .url()
    .then((url: string) => {
      if (url.includes('/block-production/overview')) {
        callback();
      }
    });
};
let slotsResponse: SlotResponse[];
let activeSlotIndex: number;

describe('BLOCK PRODUCTION OVERVIEW SLOTS', () => {
  beforeEach(() => {
    cy
      .intercept(/\/epoch\/\d+/, req => {
        req.continue(res => {
          slotsResponse = res.body;
          activeSlotIndex = slotsResponse.findIndex(slot => slot.is_current_slot);
        });
      })
      .as('slotsRequest')
      .visit(Cypress.config().baseUrl + '/block-production/overview');
  });

  it('show correct slots interval', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-slots > div:first-child')
            .then(element => {
              expect(element.text().trim()).equals(`Slots ${slotsResponse[0].global_slot} - ${slotsResponse[slotsResponse.length - 1].global_slot}`);
            });
        }
      });
  }));

  it('show 7140 slots rectangles', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-slots svg rect')
            .should('have.length', 7140);
        }
      });
  }));

  it('show correct slots colors', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          const fills: string[] = [];
          cy.get('mina-block-production-overview-slots svg rect')
            .each(element => {
              fills.push(element.attr('fill'));
            })
            .then(() => {
              expect(fills).to.have.length(7140);
              expect(
                fills
                  .map((f: string, index: number) => f === getSlotColor(index, slotsResponse[index]))
                  .every((v: boolean) => v === true),
              ).to.be.true;
            });
        }
      });
  }));
});


export enum BlockStatus {
  Empty = 'Empty',
  ToBeProduced = 'ToBeProduced',
  Orphaned = 'Orphaned',
  OrphanedPending = 'OrphanedPending',
  Canonical = 'Canonical',
  CanonicalPending = 'CanonicalPending',
  Foreign = 'Foreign',
  Missed = 'Missed',
}

function getSlotColor(i: number, slot: SlotResponse): string {
  const prefix = 'var(--';
  const suffix = ')';
  let color = 'base-container';
  if (i < activeSlotIndex && slot.block_status !== BlockStatus.Empty) {
    color = 'selected-tertiary';
  }

  if (slot.block_status === BlockStatus.Canonical || slot.block_status === BlockStatus.CanonicalPending) {
    color = 'success-primary';
  } else if (slot.block_status === BlockStatus.Orphaned || slot.block_status === BlockStatus.OrphanedPending) {
    color = 'special-selected-alt-1-primary';
  } else if (slot.block_status === BlockStatus.Missed) {
    color = 'warn-primary';
  } else if (slot.block_status === BlockStatus.ToBeProduced) {
    color = 'base-secondary';
  } else if (!(i < activeSlotIndex && slot.block_status !== BlockStatus.Empty)) {
    color = 'base-container';
  }

  if (slot.is_current_slot) {
    color = 'selected-primary';
  }
  return `${prefix}${color}${suffix}`;
}
