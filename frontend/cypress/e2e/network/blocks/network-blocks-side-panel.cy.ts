import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { NetworkBlocksState } from '@network/blocks/network-blocks.state';
import { stateSliceAsPromise } from '../../../support/commands';

const condition = (state: NetworkBlocksState) => state && state.blocks?.length > 2;
const networkBlocksState = (store: Store<MinaState>) => stateSliceAsPromise<NetworkBlocksState>(store, condition, 'network', 'blocks');


describe('NETWORK BLOCKS SIDE PANEL', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/network/blocks');
  });

  it('side panel is not open at the beginning', () => {
    cy.get('mina-network-blocks-side-panel mina-bar-graph')
      .should('not.be.visible');
  });

  it('show/hide side panel', () => {
    cy.get('mina-network-blocks-toolbar > div:nth-child(2) > button')
      .click()
      .wait(1000)
      .get('mina-bar-graph .y-grid-marks')
      .should('be.visible')
      .get('mina-network-blocks-toolbar > div:nth-child(2) > button')
      .click()
      .wait(1000)
      .get('mina-bar-graph .y-grid-marks')
      .should('not.be.visible');
  });

  it('displays correct number of bars in the bar graph', () => {
    cy.get('mina-bar-graph .histo-col')
      .should('have.length', 16)
      .get('mina-bar-graph > div > div:last-child div')
      .should('have.length', 16);
  });

  it('displays correct height of bars in the bar graph', () => {
    let yMax: number;
    let maxHeight: number;
    let thirdBarHeight: number;
    let fourthBarHeight: number;
    let fifthBarHeight: number;
    let sixthBarHeight: number;
    let tenthBarHeight: number;
    let fifteenthBarHeight: number;
    cy.window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (condition(state)) {
          cy.get('mina-bar-graph .y-tick-line > div')
            .then((ticksRefs: any) => {
              const ticks = Array.from(ticksRefs).map((t: any) => t.textContent).filter((t: string) => t !== 'Count').map(t => Number(t)).reverse();
              yMax = Math.max(...ticks) + (ticks[1] - ticks[0]);
            })
            .wait(1000)
            .get('mina-bar-graph .y-grid-marks')
            .then((marks: any) => maxHeight = marks[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(3) > div')
            .then((bar: any) => thirdBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(4) > div')
            .then((bar: any) => fourthBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(5) > div')
            .then((bar: any) => fifthBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(6) > div')
            .then((bar: any) => sixthBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(10) > div')
            .then((bar: any) => tenthBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(15) > div')
            .then((bar: any) => fifteenthBarHeight = bar[0].offsetHeight)
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                const bars = state.filteredBlocks.filter(b => b.receivedLatency || b.sentLatency).map(b => b.receivedLatency || b.sentLatency);
                const thirdBarValues = bars.filter(b => b <= 3 && b > 3 - 1);
                const fourthBarValues = bars.filter(b => b <= 4 && b > 4 - 1);
                const fifthBarValues = bars.filter(b => b <= 5 && b > 5 - 1);
                const tenthBarValues = bars.filter(b => b <= 10 && b > 10 - 1);
                const fifteenthBarValues = bars.filter(b => b <= 15 && b > 15 - 1);
                expect(thirdBarHeight).to.be.closeTo((thirdBarValues.length * maxHeight / yMax) || 4, 1);
                expect(fourthBarHeight).to.be.closeTo((fourthBarValues.length * maxHeight / yMax) || 4, 1);
                expect(fifthBarHeight).to.be.closeTo((fifthBarValues.length * maxHeight / yMax) || 4, 1);
                expect(tenthBarHeight).to.be.closeTo((tenthBarValues.length * maxHeight / yMax) || 4, 1);
                expect(fifteenthBarHeight).to.be.closeTo((fifteenthBarValues.length * maxHeight / yMax) || 4, 1);
              }
            });
        }
      });
  });

  it('displays correct height of bars in the bar graph after changing block height', () => {
    let yMax: number;
    let maxHeight: number;
    let thirdBarHeight: number;
    let fourthBarHeight: number;
    let fifthBarHeight: number;
    let sixthBarHeight: number;
    let tenthBarHeight: number;
    let fifteenthBarHeight: number;
    cy.window()
      .its('store')
      .then(networkBlocksState)
      .then((state: NetworkBlocksState) => {
        if (condition(state)) {
          cy.get('mina-network-blocks-toolbar > div:first-child .pagination-group button:first-child')
            .click({ force: true })
            .wait(1000)
            .get('mina-bar-graph .y-tick-line > div')
            .then((ticksRefs: any) => {
              const ticks = Array.from(ticksRefs).map((t: any) => t.textContent).filter((t: string) => t !== 'Count').map(t => Number(t)).reverse();
              yMax = Math.max(...ticks) + (ticks[1] - ticks[0]);
            })
            .wait(1000)
            .get('mina-bar-graph .y-grid-marks')
            .then((marks: any) => maxHeight = marks[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(3) > div')
            .then((bar: any) => thirdBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(4) > div')
            .then((bar: any) => fourthBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(5) > div')
            .then((bar: any) => fifthBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(6) > div')
            .then((bar: any) => sixthBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(10) > div')
            .then((bar: any) => tenthBarHeight = bar[0].offsetHeight)
            .get('mina-bar-graph .histo-col:nth-child(15) > div')
            .then((bar: any) => fifteenthBarHeight = bar[0].offsetHeight)
            .window()
            .its('store')
            .then(networkBlocksState)
            .then((state: NetworkBlocksState) => {
              if (condition(state)) {
                const bars = state.filteredBlocks.filter(b => b.receivedLatency || b.sentLatency).map(b => b.receivedLatency || b.sentLatency);
                const thirdBarValues = bars.filter(b => b <= 3 && b > 3 - 1);
                const fourthBarValues = bars.filter(b => b <= 4 && b > 4 - 1);
                const fifthBarValues = bars.filter(b => b <= 5 && b > 5 - 1);
                const tenthBarValues = bars.filter(b => b <= 10 && b > 10 - 1);
                const fifteenthBarValues = bars.filter(b => b <= 15 && b > 15 - 1);
                expect(thirdBarHeight).to.be.closeTo((thirdBarValues.length * maxHeight / yMax) || 4, 1);
                expect(fourthBarHeight).to.be.closeTo((fourthBarValues.length * maxHeight / yMax) || 4, 1);
                expect(fifthBarHeight).to.be.closeTo((fifthBarValues.length * maxHeight / yMax) || 4, 1);
                expect(tenthBarHeight).to.be.closeTo((tenthBarValues.length * maxHeight / yMax) || 4, 1);
                expect(fifteenthBarHeight).to.be.closeTo((fifteenthBarValues.length * maxHeight / yMax) || 4, 1);
              }
            });
        }
      });
  });
});
