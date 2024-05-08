import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { BlockProductionOverviewState } from '@block-production/overview/block-production-overview.state';
import {
  BlockProductionEpochPaginationResponse,
  SlotResponse,
} from '@block-production/overview/block-production-overview.service';

const condition = (state: BlockProductionOverviewState): boolean => state && state.epochs?.length > 0;
const getBPOverview = (store: Store<MinaState>): BlockProductionOverviewState => stateSliceAsPromise<BlockProductionOverviewState>(store, condition, 'blockProduction', 'overview');
const execute = (callback: () => void) => {
  cy.wait('@slotsRequest')
    .wait('@epochDetailsRequest')
    .url()
    .then((url: string) => {
      if (url.includes('/block-production/overview')) {
        callback();
      }
    });
};
let slotsResponse: SlotResponse[];
let epochDetails: BlockProductionEpochPaginationResponse;
let activeSlotIndex: number;

describe('BLOCK PRODUCTION OVERVIEW TOOLBAR', () => {
  beforeEach(() => {
    cy
      .intercept(/\/epoch\/summary\/(?!.*\?limit=\d+)(latest|\d+)/, (req) => {
        req.continue(res => {
          if (!Array.isArray(res.body)) {
            res.body = [res.body];
          }
          epochDetails = res.body[0];
        });
      })
      .as('epochDetailsRequest')
      .intercept(/\/epoch\/\d+/, (req) => {
        req.continue(res => {
          slotsResponse = res.body;
          activeSlotIndex = slotsResponse.findIndex(slot => slot.is_current_slot);
        });
      })
      .as('slotsRequest')
      .visit(Cypress.config().baseUrl + '/block-production/overview');
  });

  it('show correct epoch number in pagination', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group > div')
            .should('have.text', `${epochDetails.epoch_number}`);
        }
      });
  }));

  it('show next epoch page disabled', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:last-child')
            .should('have.class', 'disabled');
        }
      });
  }));

  it('go to previous epoch', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
            .then(button => {
              if (!button.hasClass('disabled')) {
                cy.wrap(button)
                  .click()
                  .wait('@epochDetailsRequest')
                  .then(() => {
                    cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group > div')
                      .should('have.text', `${epochDetails.epoch_number}`)
                      .window()
                      .its('store')
                      .then(getBPOverview)
                      .then((state: BlockProductionOverviewState) => {
                        if (condition(state)) {
                          expect(state.activeEpoch.epochNumber).to.equal(epochDetails.epoch_number);
                        }
                      })
                      .get('mina-block-production-overview-epoch-graphs .active-epoch .title')
                      .then(el => expect(el.text().trim()).to.equal(epochDetails.epoch_number.toString()));
                  });
              }
            });
        }
      });
  }));

  it('go 2 epochs before', () => execute(() => {
    const initialEpoch = epochDetails.epoch_number;
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.epochs.length > 2) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
            .then(button => {
              if (!button.hasClass('disabled')) {
                cy.wrap(button)
                  .click()
                  .wait('@epochDetailsRequest')
                  .then(() => {
                    cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
                      .then(button2 => {
                        if (!button2.hasClass('disabled')) {
                          cy.wait(2000)
                            .wrap(button2)
                            .click()
                            .wait('@epochDetailsRequest')
                            .then(() => {
                              cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group > div')
                                .should('have.text', `${epochDetails.epoch_number}`)
                                .window()
                                .its('store')
                                .then(getBPOverview)
                                .then((state: BlockProductionOverviewState) => {
                                  if (condition(state)) {
                                    expect(state.activeEpoch.epochNumber).to.equal(epochDetails.epoch_number);
                                    expect(state.activeEpoch.epochNumber).to.equal(initialEpoch - 2);
                                  }
                                })
                                .get('mina-block-production-overview-epoch-graphs .active-epoch .title')
                                .then(el => {
                                  expect(el.text().trim()).to.equal(epochDetails.epoch_number.toString());
                                  expect(el.text().trim()).to.equal((initialEpoch - 2).toString());
                                });
                            });
                        }
                      });
                  });
              }
            });
        }
      });
  }));

  it('go 2 epochs before and 1 after', () => execute(() => {
    const initialEpoch = epochDetails.epoch_number;
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.epochs.length > 2) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
            .then(button => {
              if (!button.hasClass('disabled')) {
                cy.wrap(button)
                  .click()
                  .wait('@epochDetailsRequest')
                  .then(() => {
                    cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
                      .then(button2 => {
                        if (!button2.hasClass('disabled')) {
                          cy.wait(2000)
                            .wrap(button2)
                            .click()
                            .wait('@epochDetailsRequest')
                            .then(() => {
                              cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:last-child')
                                .then(button3 => {
                                  if (!button3.hasClass('disabled')) {
                                    cy.wait(2000)
                                      .wrap(button3)
                                      .click()
                                      .wait('@epochDetailsRequest')
                                      .then(() => {
                                        cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group > div')
                                          .should('have.text', `${epochDetails.epoch_number}`)
                                          .window()
                                          .its('store')
                                          .then(getBPOverview)
                                          .then((state: BlockProductionOverviewState) => {
                                            if (condition(state)) {
                                              expect(state.activeEpoch.epochNumber).to.equal(epochDetails.epoch_number);
                                              expect(state.activeEpoch.epochNumber).to.equal(initialEpoch - 1);
                                            }
                                          })
                                          .get('mina-block-production-overview-epoch-graphs .active-epoch .title')
                                          .then(el => {
                                            expect(el.text().trim()).to.equal(epochDetails.epoch_number.toString());
                                            expect(el.text().trim()).to.equal((initialEpoch - 1).toString());
                                          });
                                      });
                                  }
                                });
                            });
                        }
                      });
                  });
              }
            });
        }
      });
  }));

  it('go 2 epochs before and then to last epoch', () => execute(() => {
    const initialEpoch = epochDetails.epoch_number;
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.epochs.length > 2) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
            .then(button => {
              if (!button.hasClass('disabled')) {
                cy.wrap(button)
                  .click()
                  .wait('@epochDetailsRequest')
                  .then(() => {
                    cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
                      .then(button2 => {
                        if (!button2.hasClass('disabled')) {
                          cy.wait(2000)
                            .wrap(button2)
                            .click()
                            .wait('@epochDetailsRequest')
                            .then(() => {
                              cy.get('mina-block-production-overview-toolbar mina-pagination > button:last-child')
                                .should('not.have.class', 'disabled')
                                .then(btnLastEpoch => {
                                  if (!btnLastEpoch.hasClass('disabled')) {
                                    cy.wait(2000)
                                      .wrap(btnLastEpoch)
                                      .click()
                                      .wait('@epochDetailsRequest')
                                      .then(() => {
                                        cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group > div')
                                          .should('have.text', `${epochDetails.epoch_number}`)
                                          .window()
                                          .its('store')
                                          .then(getBPOverview)
                                          .then((state: BlockProductionOverviewState) => {
                                            if (condition(state)) {
                                              expect(state.activeEpoch.epochNumber).to.equal(epochDetails.epoch_number);
                                              expect(state.activeEpoch.epochNumber).to.equal(initialEpoch);
                                            }
                                          })
                                          .get('mina-block-production-overview-epoch-graphs .active-epoch .title')
                                          .then(el => {
                                            expect(el.text().trim()).to.equal(epochDetails.epoch_number.toString());
                                            expect(el.text().trim()).to.equal((initialEpoch).toString());
                                          });
                                      });
                                  }
                                });
                            });
                        }
                      });
                  });
              }
            });
        }
      });
  }));

  it('show correct slots interval when changing active epoch', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
            .then(button => {
              if (!button.hasClass('disabled')) {
                cy.get('mina-block-production-overview-slots > div:first-child')
                  .then(element => {
                    expect(element.text().trim()).equals(`Slots ${slotsResponse[0].global_slot} - ${slotsResponse[slotsResponse.length - 1].global_slot}`);
                  })
                  .wrap(button)
                  .click()
                  .wait('@epochDetailsRequest')
                  .wait('@slotsRequest')
                  .then(() => {
                    cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group > div')
                      .should('have.text', `${epochDetails.epoch_number}`)
                      .window()
                      .its('store')
                      .then(getBPOverview)
                      .then((state: BlockProductionOverviewState) => {
                        if (condition(state)) {
                          expect(state.activeEpoch.epochNumber).to.equal(epochDetails.epoch_number);

                          if (state.activeEpoch.slots.length) {
                            cy.get('mina-block-production-overview-slots > div:first-child')
                              .then(element => {
                                expect(element.text().trim()).equals(`Slots ${slotsResponse[0].global_slot} - ${slotsResponse[slotsResponse.length - 1].global_slot}`);
                              });
                          }
                        }
                      });
                  });
              }
            });
        }
      });
  }));

  it('show correct side panel data after changing epoch', () => execute(() => {
    const initialEpoch = epochDetails.epoch_number;
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
            .then(button => {
              if (!button.hasClass('disabled')) {
                cy.get('mina-block-production-overview-side-panel .tab:first-child')
                  .then(el => {
                    expect(el.text().trim()).equals(`Epoch ${epochDetails.epoch_number}`);
                  })
                  .wrap(button)
                  .click()
                  .wait('@epochDetailsRequest')
                  .wait('@slotsRequest')
                  .then(() => {
                    cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group > div')
                      .should('have.text', `${epochDetails.epoch_number}`)
                      .window()
                      .its('store')
                      .then(getBPOverview)
                      .then((state: BlockProductionOverviewState) => {
                        if (condition(state)) {

                          const stats = {
                            totalSlots: state.activeEpoch.details.totalSlots,
                            canonical: state.activeEpoch.details.canonical,
                            orphaned: state.activeEpoch.details.orphaned,
                            missed: state.activeEpoch.details.missed,
                            futureRights: state.activeEpoch.details.futureRights,
                            earnedRewards: state.activeEpoch.details.earnedRewards,
                            expectedRewards: state.activeEpoch.details.expectedRewards,
                            balanceProducer: state.activeEpoch.details.balanceProducer,
                            balanceDelegated: state.activeEpoch.details.balanceDelegated,
                            balanceStaked: state.activeEpoch.details.balanceStaked,
                          };

                          cy.get('mina-block-production-overview-side-panel .tab:first-child')
                            .then(el => {
                              expect(el.text().trim()).equals(`Epoch ${epochDetails.epoch_number}`);
                              expect(state.activeEpoch.epochNumber).to.equal(epochDetails.epoch_number);
                              expect(el.text().trim()).equals(`Epoch ${initialEpoch - 1}`);
                              expect(state.activeEpoch.epochNumber).to.equal(initialEpoch - 1);
                            })
                            .get('mina-block-production-overview-side-panel .bar > div:nth-child(1)')
                            .then(el => {
                              const widthPercentage = ((stats.canonical / stats.totalSlots) * 100).toString().slice(0, 3);
                              expect(el.attr('style')).to.satisfy((style: string) => {
                                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
                              });
                            })
                            .get('mina-block-production-overview-side-panel .bar > div:nth-child(2)')
                            .then(el => {
                              const widthPercentage = ((stats.orphaned / stats.totalSlots) * 100).toString().slice(0, 3);
                              expect(el.attr('style')).to.satisfy((style: string) => {
                                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
                              });
                            })
                            .get('mina-block-production-overview-side-panel .bar > div:nth-child(3)')
                            .then(el => {
                              const widthPercentage = ((stats.missed / stats.totalSlots) * 100).toString().slice(0, 3);
                              expect(el.attr('style')).to.satisfy((style: string) => {
                                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
                              });
                            })
                            .get('mina-block-production-overview-side-panel .bar > div:nth-child(4)')
                            .then(el => {
                              const widthPercentage = ((stats.futureRights / stats.totalSlots) * 100).toString().slice(0, 3);
                              expect(el.attr('style')).to.satisfy((style: string) => {
                                return style.includes(`width: ${widthPercentage}`) || style.includes('width: 0%');
                              });
                            })
                            .get('mina-block-production-overview-side-panel .bar + div > span')
                            .should('have.text', `${stats.canonical}`)
                            .get('mina-block-production-overview-side-panel .bar + div + div > span')
                            .should('have.text', `${stats.orphaned}`)
                            .get('mina-block-production-overview-side-panel .bar + div + div + div > span')
                            .should('have.text', `${stats.missed}`)
                            .get('mina-block-production-overview-side-panel .bar + div + div + div + div > span')
                            .should('have.text', `${stats.futureRights}`)
                            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(3) div')
                            .should('have.text', `Expected ${stats.expectedRewards} Mina`)
                            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(4) div')
                            .should('have.text', `Earned ${stats.earnedRewards} Mina`)
                            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(6) div')
                            .should('have.text', `Producer ${stats.balanceProducer} Mina`)
                            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(7) div')
                            .should('have.text', `Delegated ${stats.balanceDelegated} Mina`)
                            .get('mina-block-production-overview-side-panel .h-minus-lg > div:nth-child(8) div')
                            .should('have.text', `Staked ${stats.balanceStaked} Mina`);
                        }
                      });
                  });
              }
            });
        }
      });
  }));

  it('show correct slots colors in the slots map', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state)) {
          cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group button:first-child')
            .then(button => {
              if (!button.hasClass('disabled')) {
                cy.wrap(button)
                  .click()
                  .wait('@epochDetailsRequest')
                  .wait('@slotsRequest')
                  .then(() => {
                    cy.get('mina-block-production-overview-toolbar mina-pagination .pagination-group > div')
                      .should('have.text', `${epochDetails.epoch_number}`)
                      .window()
                      .its('store')
                      .then(getBPOverview)
                      .then((state: BlockProductionOverviewState) => {
                        if (condition(state)) {
                          expect(state.activeEpoch.epochNumber).to.equal(epochDetails.epoch_number);

                          if (state.activeEpoch.slots.length) {
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
                        }
                      });
                  });
              }
            });
        }
      });
  }));

  it('disable canonical blocks filter', () => execute(() => {
    cy.get('mina-block-production-overview-toolbar mina-pagination + div')
      .should('have.class', 'bg-success-container')
      .click()
      .wait(200)
      .should('not.have.class', 'bg-success-container')
      .window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.activeEpoch.slots.length) {
          const fills: string[] = [];
          cy.get('mina-block-production-overview-slots svg rect')
            .each(element => {
              fills.push(element.attr('fill'));
            })
            .then(() => {
              expect(fills).to.have.length(7140);
              expect(fills.every(f => f !== 'var(--success-primary)')).to.be.true;
              expect(state.filters.canonical).to.be.false;
              expect(
                fills
                  .map((f: string, index: number) => f === getSlotColor(index, slotsResponse[index], state.filters))
                  .every((v: boolean) => v === true),
              ).to.be.true;
            });
        }
      });
  }));

  it('disable orphaned blocks filter', () => execute(() => {
    cy.get('mina-block-production-overview-toolbar mina-pagination + div + div')
      .should('have.class', 'bg-special-selected-alt-1-container')
      .click()
      .wait(200)
      .should('not.have.class', 'bg-special-selected-alt-1-container')
      .window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.activeEpoch.slots.length) {
          const fills: string[] = [];
          cy.get('mina-block-production-overview-slots svg rect')
            .each(element => {
              fills.push(element.attr('fill'));
            })
            .then(() => {
              expect(fills).to.have.length(7140);
              expect(fills.every(f => f !== 'var(--special-selected-alt-1-primary)')).to.be.true;
              expect(state.filters.orphaned).to.be.false;
              expect(
                fills
                  .map((f: string, index: number) => f === getSlotColor(index, slotsResponse[index], state.filters))
                  .every((v: boolean) => v === true),
              ).to.be.true;
            });
        }
      });
  }));

  it('disable missed blocks filter', () => execute(() => {
    cy.get('mina-block-production-overview-toolbar mina-pagination + div + div + div')
      .should('have.class', 'bg-warn-container')
      .click()
      .wait(200)
      .should('not.have.class', 'bg-warn-container')
      .window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.activeEpoch.slots.length) {
          const fills: string[] = [];
          cy.get('mina-block-production-overview-slots svg rect')
            .each(element => {
              fills.push(element.attr('fill'));
            })
            .then(() => {
              expect(fills).to.have.length(7140);
              expect(fills.every(f => f !== 'var(--warn-primary)')).to.be.true;
              expect(state.filters.missed).to.be.false;
              expect(
                fills
                  .map((f: string, index: number) => f === getSlotColor(index, slotsResponse[index], state.filters))
                  .every((v: boolean) => v === true),
              ).to.be.true;
            });
        }
      });
  }));

  it('disable future rights filter', () => execute(() => {
    cy.get('mina-block-production-overview-toolbar mina-pagination + div + div + div + div')
      .should('have.class', 'primary')
      .should('not.have.class', 'tertiary')
      .click()
      .wait(200)
      .should('not.have.class', 'primary')
      .should('have.class', 'tertiary')
      .window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.activeEpoch.slots.length) {
          const fills: string[] = [];
          cy.get('mina-block-production-overview-slots svg rect')
            .each(element => {
              fills.push(element.attr('fill'));
            })
            .then(() => {
              expect(fills).to.have.length(7140);
              expect(fills.every(f => f !== 'var(--base-secondary)')).to.be.true;
              expect(state.filters.future).to.be.false;
              expect(
                fills
                  .map((f: string, index: number) => f === getSlotColor(index, slotsResponse[index], state.filters))
                  .every((v: boolean) => v === true),
              ).to.be.true;
            });
        }
      });
  }));

  it('disable all filters', () => execute(() => {
    cy
      .get('mina-block-production-overview-toolbar mina-pagination + div')
      .click()
      .wait(500)
      .get('mina-block-production-overview-toolbar mina-pagination + div + div')
      .click()
      .wait(500)
      .get('mina-block-production-overview-toolbar mina-pagination + div + div + div')
      .click()
      .wait(500)
      .get('mina-block-production-overview-toolbar mina-pagination + div + div + div + div')
      .click()
      .wait(500)
      .window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.activeEpoch.slots.length) {
          const fills: string[] = [];
          cy.get('mina-block-production-overview-slots svg rect')
            .each(element => {
              fills.push(element.attr('fill'));
            })
            .then(() => {
              expect(fills).to.have.length(7140);
              expect(fills.every(f => f !== 'var(--base-secondary)' && f !== 'var(--warn-primary)' && f !== 'var(--special-selected-alt-1-primary)' && f !== 'var(--success-primary)')).to.be.true;
              expect(state.filters.canonical).to.be.false;
              expect(state.filters.orphaned).to.be.false;
              expect(state.filters.missed).to.be.false;
              expect(state.filters.future).to.be.false;
              expect(
                fills
                  .map((f: string, index: number) => f === getSlotColor(index, slotsResponse[index], state.filters))
                  .every((v: boolean) => v === true),
              ).to.be.true;
            });
        }
      });
  }));

  it('show correct filters numbers', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.activeEpoch.slots.length) {
          cy.get('mina-block-production-overview-toolbar mina-pagination + div')
            .should('have.text', `${state.activeEpoch.details.canonical} Canonical Blocks `)
            .get('mina-block-production-overview-toolbar mina-pagination + div + div')
            .should('have.text', `${state.activeEpoch.details.orphaned} Orphaned Blocks `)
            .get('mina-block-production-overview-toolbar mina-pagination + div + div + div')
            .should('have.text', `${state.activeEpoch.details.missed} Missed Blocks `)
            .get('mina-block-production-overview-toolbar mina-pagination + div + div + div + div')
            .should('have.text', `${state.activeEpoch.details.futureRights} Future Rights `);
        }
      });
  }));

  it('change scale for epoch graphs', () => execute(() => {
    cy.window()
      .its('store')
      .then(getBPOverview)
      .then((state: BlockProductionOverviewState) => {
        if (condition(state) && state.activeEpoch.slots.length) {
          let heightOfFirstWindow = 0;
          cy.get('.cdk-overlay-container #cdk-overlay-0')
            .should('not.exist')
            .get('mina-block-production-overview-toolbar .flex-between.w-100 > div:last-child')
            .should('include.text', 'adaptive')
            .should('include.text', 'Scale')
            .click()
            .get('.cdk-overlay-container #cdk-overlay-0')
            .should('exist')
            .get('.cdk-overlay-container #cdk-overlay-0 .bg-container-hover')
            .should('have.length', 2)
            .get('mina-block-production-overview-epoch-graphs .active-epoch .positive > div:first-child')
            .then(el => {
              heightOfFirstWindow = el[0].getBoundingClientRect().height;
            })
            .get('.cdk-overlay-container #cdk-overlay-0 .bg-container-hover:last-child')
            .click()
            .wait(500)
            .get('mina-block-production-overview-epoch-graphs .active-epoch .positive > div:first-child')
            .then(el => {
              expect(el[0].getBoundingClientRect().height).to.be.lte(heightOfFirstWindow);
            })
            .window()
            .its('store')
            .then(getBPOverview)
            .then((state: BlockProductionOverviewState) => {
              if (condition(state)) {
                expect(state.scale).to.equal('linear');
              }
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

function getSlotColor(i: number, slot: SlotResponse, filters = {
  canonical: true,
  orphaned: true,
  missed: true,
  future: true,
}): string {
  const prefix = 'var(--';
  const suffix = ')';
  let color = 'base-container';
  if (i < activeSlotIndex && slot.block_status !== BlockStatus.Empty) {
    color = 'selected-tertiary';
  }

  if (slot.block_status === BlockStatus.Canonical || slot.block_status === BlockStatus.CanonicalPending) {
    color = 'success-' + (filters?.canonical ? 'primary' : 'tertiary');
  } else if (slot.block_status === BlockStatus.Orphaned || slot.block_status === BlockStatus.OrphanedPending) {
    color = 'special-selected-alt-1-' + (filters?.orphaned ? 'primary' : 'tertiary');
  } else if (slot.block_status === BlockStatus.Missed) {
    color = 'warn-' + (filters?.missed ? 'primary' : 'tertiary');
  } else if (slot.block_status === BlockStatus.ToBeProduced) {
    color = 'base-' + (filters?.future ? 'secondary' : 'divider');
  } else if (!(i < activeSlotIndex && slot.block_status !== BlockStatus.Empty)) {
    color = 'base-container';
  }

  if (slot.is_current_slot) {
    color = 'selected-primary';
  }
  return `${prefix}${color}${suffix}`;
}
