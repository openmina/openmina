import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { MemoryResourcesState } from '@resources/memory/memory-resources.state';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';

const condition = (state: MemoryResourcesState): boolean => !!state?.resource;
const getResources = (store: Store<MinaState>): MemoryResourcesState => stateSliceAsPromise<MemoryResourcesState>(store, condition, 'resources', 'memory');

describe('MEMORY RESOURCES TREEMAP', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/resources/memory');
  });

  it('display root children in the treemap', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          expect(state.resource.children.length).above(0);
          expect(state.resource).to.equal(state.activeResource);
          cy.get('app-memory-resources-treemap svg .treemap')
            .find('g')
            .should('have.length', state.resource.children.length);
        }
      });
  });

  it('display correct value', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          state.activeResource.children
            .slice(0, state.activeResource.children.length)
            .forEach((child: MemoryResource, i: number) => {
              cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${i + 1})`)
                .then((g: any) => {
                  const text = g.find('text.name').text().trim();
                  const val = g.find('text.val').text().trim();
                  expect(text).equals(child.name.executableName);
                  expect(val).equals(transform(child.value));
                });
            });
        }
      });
  });

  it('display child count as expected', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          state.activeResource.children
            .slice(0, state.activeResource.children.length)
            .forEach((child: MemoryResource, i: number) => {
              cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${i + 1})`)
                .then((g: any) => {
                  const rectFill = g.find('rect:first-child').attr('fill');
                  expect(rectFill).equals(child.children.length > 0 ? 'var(--base-surface-top)' : 'var(--base-background)');
                  if (child.children.length > 0) {
                    expect(g.attr('cursor')).equals('pointer');
                  } else {
                    expect(g.attr('cursor')).equals(undefined);
                  }
                });
            });
        }
      });
  });

  it('click on an chunk to show it\'s children', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            const expectedActiveResourceName = state.activeResource.children[rowIndex].name.executableName;
            cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${rowIndex + 1})`)
              .click()
              .wait(500)
              .get('app-memory-resources-treemap svg .treemap g')
              .each((g: any, i: number) => {
                expect(g.find('text.name').text().trim()).equals(state.activeResource.children[rowIndex].children[i].name.executableName);
                expect(g.find('text.val').text().trim()).equals(transform(state.activeResource.children[rowIndex].children[i].value));
              })
              .window()
              .its('store')
              .then(getResources)
              .then((state: MemoryResourcesState) => {
                expect(state.activeResource.name.executableName).equals(expectedActiveResourceName);
              });
          }
        }
      });
  });

  it('click on a chunk without children will do nothing', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length === 0);
          if (rowIndex >= 0) {
            const expectedActiveResourceName = state.activeResource.name.executableName;
            cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${rowIndex + 1})`)
              .click()
              .wait(500)
              .get('app-memory-resources-treemap svg .treemap g')
              .each((g: any, i: number) => {
                expect(g.find('text.name').text().trim()).equals(state.activeResource.children[i].name.executableName);
                expect(g.find('text.val').text().trim()).equals(transform(state.activeResource.children[i].value));
              })
              .window()
              .its('store')
              .then(getResources)
              .then((state: MemoryResourcesState) => {
                expect(state.activeResource.name.executableName).equals(expectedActiveResourceName);
              });
          }
        }
      });
  });

  it('click on a chunk will show it as the active one', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${rowIndex + 1})`)
              .click()
              .wait(500)
              .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(1) div')
              .then((span: any) => expect(span.text().trim()).equals(state.activeResource.children[rowIndex].name.executableName + ' ' + transform(state.activeResource.children[rowIndex].value)));
          }
        }
      });
  });

  it('click on a chunk will add that item in the breadcrumbs', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${rowIndex + 1})`)
              .click()
              .wait(500)
              .get('app-memory-resources nav span.breadcrumb')
              .should('have.length', 2)
              .get('app-memory-resources nav span.breadcrumb:last-child')
              .then((span: any) => expect(span.text().trim()).equals(state.activeResource.children[rowIndex].name.executableName));
          }
        }
      });
  });

  it('click on a chunk will show the children in the table', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${rowIndex + 1})`)
              .click()
              .wait(500)
              .get('app-memory-resources-table .mina-table .row:not(.head)')
              .each((row: any, i: number) => {
                expect(row.find('span:nth-child(3)').text().trim()).equals(state.activeResource.children[rowIndex].children[i].name.executableName);
                expect(row.find('span:nth-child(4)').text().trim()).equals(transform(state.activeResource.children[rowIndex].children[i].value));
              });
          }
        }
      });
  });

  it('pressing the back arrow will go back one level', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${rowIndex + 1})`)
              .click()
              .wait(500)
              .get('app-memory-resources-treemap svg .treemap g')
              .each((g: any, i: number) => {
                expect(g.find('text.name').text().trim()).equals(state.activeResource.children[rowIndex].children[i].name.executableName);
                expect(g.find('text.val').text().trim()).equals(transform(state.activeResource.children[rowIndex].children[i].value));
              })
              .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(1) > span')
              .click()
              .wait(1500)
              .window()
              .its('store')
              .then(getResources)
              .then((state: MemoryResourcesState) => {
                if (condition(state)) {
                  cy.get('app-memory-resources-treemap svg .treemap g')
                    .each((g: any, i: number) => {
                      expect(g.find('text.name').text().trim()).equals(state.activeResource.children[i].name.executableName);
                      expect(g.find('text.val').text().trim()).equals(transform(state.activeResource.children[i].value));
                    });
                }
              });
          }
        }
      });
  });

  it('pressing escape will go back one level', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            cy.get(`app-memory-resources-treemap svg .treemap g:nth-child(${rowIndex + 1})`)
              .click()
              .wait(500)
              .get('app-memory-resources-treemap svg .treemap g')
              .each((g: any, i: number) => {
                expect(g.find('text.name').text().trim()).equals(state.activeResource.children[rowIndex].children[i].name.executableName);
                expect(g.find('text.val').text().trim()).equals(transform(state.activeResource.children[rowIndex].children[i].value));
              })
              .wait(1000)
              .get('body')
              .type('{esc}')
              .wait(1500)
              .window()
              .its('store')
              .then(getResources)
              .then((state: MemoryResourcesState) => {
                if (condition(state)) {
                  cy.get('app-memory-resources-treemap svg .treemap g')
                    .each((g: any, i: number) => {
                      expect(g.find('text.name').text().trim()).equals(state.activeResource.children[i].name.executableName);
                      expect(g.find('text.val').text().trim()).equals(transform(state.activeResource.children[i].value));
                    });
                }
              });
          }
        }
      });
  });
});

function transform(kilobytes: number): string {
  if (kilobytes >= 1048576) {
    return `${(kilobytes / 1048576).toFixed(2)} GB`;
  } else if (kilobytes >= 1024) {
    return `${(kilobytes / 1024).toFixed(2)} MB`;
  }
  return `${kilobytes?.toFixed(2) || 0} KB`;
}
