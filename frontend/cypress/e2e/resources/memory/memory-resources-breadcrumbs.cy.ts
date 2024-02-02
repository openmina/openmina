import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { MemoryResourcesState } from '@resources/memory/memory-resources.state';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';

const condition = (state: MemoryResourcesState): boolean => !!state?.resource;
const getResources = (store: Store<MinaState>): MemoryResourcesState => stateSliceAsPromise<MemoryResourcesState>(store, condition, 'resources', 'memory');

describe('MEMORY RESOURCES BREADCRUMBS', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/resources/memory');
  });

  it('at the beginning there should be only one breadcrumb', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.get('app-memory-resources nav .breadcrumb')
            .should('have.length', 1)
            .should('have.css', 'cursor', 'auto');
        }
      });
  });

  it('click on a row will increase breadcrumbs by one', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-table .row[idx="${rowIndex}"]`)
              .click()
              .wait(500)
              .get('app-memory-resources nav .breadcrumb')
              .should('have.length', 2)
              .each((breadcrumb: JQuery<HTMLDivElement>, index: number) => {
                if (index === 0) {
                  expect(breadcrumb.text()).equals('root');
                } else {
                  expect(breadcrumb.text()).equals(state.activeResource.children[rowIndex].name.executableName);
                }
              });
          }
        }
      });
  });

  it('click on 2 chunks will result in 3 breadcrumbs', () => {
    let once = false;
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (!once && condition(state)) {
          once = true;
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-table .row[idx="${rowIndex}"]`)
              .click()
              .wait(500)
              .window()
              .its('store')
              .then(getResources)
              .then((state2: MemoryResourcesState) => {
                if (condition(state2)) {
                  const rowIndex2 = state2.activeResource.children.findIndex(c => c.children.length > 0);
                  if (rowIndex2 >= 0) {
                    cy.get(`app-memory-resources-table .row[idx="${rowIndex2}"]`)
                      .click()
                      .wait(500)
                      .get('app-memory-resources nav .breadcrumb')
                      .should('have.length', 3)
                      .each((breadcrumb: JQuery<HTMLDivElement>, index: number) => {
                        if (index === 0) {
                          expect(breadcrumb.text()).equals('root');
                        } else if (index === 1) {
                          expect(breadcrumb.text()).equals(state.activeResource.children[rowIndex].name.executableName);
                        } else {
                          expect(breadcrumb.text()).equals(state.activeResource.children[rowIndex].children[rowIndex2].name.executableName);
                        }
                      });
                  }
                }
              });
          }
        }
      });
  });

  it('click on 2 chunks and then back will result in 2 breadcrumbs', () => {
    let once = false;
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (!once && condition(state)) {
          once = true;
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-table .row[idx="${rowIndex}"]`)
              .click()
              .wait(500)
              .window()
              .its('store')
              .then(getResources)
              .then((state2: MemoryResourcesState) => {
                if (condition(state2)) {
                  const rowIndex2 = state2.activeResource.children.findIndex(c => c.children.length > 0);
                  if (rowIndex2 >= 0) {
                    cy.get(`app-memory-resources-table .row[idx="${rowIndex2}"]`)
                      .click()
                      .wait(500)
                      .get('app-memory-resources nav .breadcrumb')
                      .should('have.length', 3)
                      .each((breadcrumb: JQuery<HTMLDivElement>, index: number) => {
                        if (index === 0) {
                          expect(breadcrumb.text()).equals('root');
                        } else if (index === 1) {
                          expect(breadcrumb.text()).equals(state.activeResource.children[rowIndex].name.executableName);
                        } else {
                          expect(breadcrumb.text()).equals(state.activeResource.children[rowIndex].children[rowIndex2].name.executableName);
                        }
                      })
                      .get('app-memory-resources nav .breadcrumb')
                      .eq(1)
                      .click()
                      .wait(500)
                      .get('app-memory-resources nav .breadcrumb')
                      .should('have.length', 2)
                      .each((breadcrumb: JQuery<HTMLDivElement>, index: number) => {
                        if (index === 0) {
                          expect(breadcrumb.text()).equals('root');
                        } else {
                          expect(breadcrumb.text()).equals(state.activeResource.children[rowIndex].name.executableName);
                        }
                      });
                  }
                }
              });
          }
        }
      });
  });

  it('click on 2 chunks and then on root will result in 1 breadcrumb', () => {
    let once = false;
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (!once && condition(state)) {
          once = true;
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-table .row[idx="${rowIndex}"]`)
              .click()
              .wait(500)
              .window()
              .its('store')
              .then(getResources)
              .then((state2: MemoryResourcesState) => {
                if (condition(state2)) {
                  const rowIndex2 = state2.activeResource.children.findIndex(c => c.children.length > 0);
                  if (rowIndex2 >= 0) {
                    cy.get(`app-memory-resources-table .row[idx="${rowIndex2}"]`)
                      .click()
                      .wait(500)
                      .get('app-memory-resources nav .breadcrumb')
                      .should('have.length', 3)
                      .each((breadcrumb: JQuery<HTMLDivElement>, index: number) => {
                        if (index === 0) {
                          expect(breadcrumb.text()).equals('root');
                        } else if (index === 1) {
                          expect(breadcrumb.text()).equals(state.activeResource.children[rowIndex].name.executableName);
                        } else {
                          expect(breadcrumb.text()).equals(state.activeResource.children[rowIndex].children[rowIndex2].name.executableName);
                        }
                      })
                      .get('app-memory-resources nav .breadcrumb')
                      .first()
                      .click()
                      .wait(500)
                      .get('app-memory-resources nav .breadcrumb')
                      .should('have.length', 1)
                      .each((breadcrumb: JQuery<HTMLDivElement>) => {
                        expect(breadcrumb.text()).equals('root');
                      });
                  }
                }
              });
          }
        }
      });
  });
});
