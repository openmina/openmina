import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { MemoryResourcesState } from '@resources/memory/memory-resources.state';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';

const condition = (state: MemoryResourcesState): boolean => !!state?.resource;
const getResources = (store: Store<MinaState>): MemoryResourcesState => stateSliceAsPromise<MemoryResourcesState>(store, condition, 'resources', 'memory');

describe('MEMORY RESOURCES TOOLBAR', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/resources/memory');
  });

  it('selecting 1024 granularity will trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=1024&reverse=false')
            .as('getResources')
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(1)')
            .click()
            .wait('@getResources')
            .window()
            .its('store')
            .then(getResources)
            .then((state: MemoryResourcesState) => {
              expect(state.granularity).equals(1024);
            });
        }
      });
  });

  it('selecting 512 granularity will not trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=512&reverse=false', cy.spy().as('getResources'))
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(2)')
            .click()
            .get('@getResources').should("not.have.been.called")
        }
      });
  });

  it('selecting 256 granularity will trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=256&reverse=false')
            .as('getResources')
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(3)')
            .click()
            .wait('@getResources')
            .window()
            .its('store')
            .then(getResources)
            .then((state: MemoryResourcesState) => {
              expect(state.granularity).equals(256);
            });
        }
      });
  });

  it('selecting 128 granularity will trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=128&reverse=false')
            .as('getResources')
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(4)')
            .click()
            .wait('@getResources')
            .window()
            .its('store')
            .then(getResources)
            .then((state: MemoryResourcesState) => {
              expect(state.granularity).equals(128);
            });
        }
      });
  });

  it('selecting 64 granularity will trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=64&reverse=false')
            .as('getResources')
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(5)')
            .click()
            .wait('@getResources')
            .window()
            .its('store')
            .then(getResources)
            .then((state: MemoryResourcesState) => {
              expect(state.granularity).equals(64);
            });
        }
      });
  });

  it('selecting 32 granularity will trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=32&reverse=false')
            .as('getResources')
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(6)')
            .click()
            .wait('@getResources')
            .window()
            .its('store')
            .then(getResources)
            .then((state: MemoryResourcesState) => {
              expect(state.granularity).equals(32);
            });
        }
      });
  });

  it('selecting 16 granularity will trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=16&reverse=false')
            .as('getResources')
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(7)')
            .click()
            .wait('@getResources')
            .window()
            .its('store')
            .then(getResources)
            .then((state: MemoryResourcesState) => {
              expect(state.granularity).equals(16);
            });
        }
      });
  });

  it('selecting 8 granularity will trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=8&reverse=false')
            .as('getResources')
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(8)')
            .click()
            .wait('@getResources')
            .window()
            .its('store')
            .then(getResources)
            .then((state: MemoryResourcesState) => {
              expect(state.granularity).equals(8);
            });
        }
      });
  });

  it('selecting 4 granularity will trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree?threshold=4&reverse=false')
            .as('getResources')
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(2)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak .h-md:nth-child(9)')
            .click()
            .wait('@getResources')
            .window()
            .its('store')
            .then(getResources)
            .then((state: MemoryResourcesState) => {
              expect(state.granularity).equals(4);
            });
        }
      });
  });

  it('selecting a new treemap view will not trigger a new http request', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.intercept('GET', '/v1/tree**', cy.spy().as('getResources'))
            .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(2) > button:nth-child(4)')
            .click()
            .get('.cdk-overlay-container .popup-box-shadow-weak div:nth-child(4)')
            .click()
            .get('@getResources').should("not.have.been.called")
        }
      });
  });
});
