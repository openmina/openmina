import { Store } from '@ngrx/store';
import { MinaState } from '@app/app.setup';
import { stateSliceAsPromise } from '../../../support/commands';
import { MemoryResourcesState } from '@resources/memory/memory-resources.state';
import { MemoryResource } from '@shared/types/resources/memory/memory-resource.type';

const condition = (state: MemoryResourcesState): boolean => !!state?.resource;
const getResources = (store: Store<MinaState>): MemoryResourcesState => stateSliceAsPromise<MemoryResourcesState>(store, condition, 'resources', 'memory');

describe('MEMORY RESOURCES TABLE', () => {
  beforeEach(() => {
    cy.visit(Cypress.config().baseUrl + '/resources/memory');
  });

  it('should have correct title', () => {
    cy.wait(2000)
      .window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          cy.get('mina-toolbar span')
            .then((span: any) => expect(span).contain('Resources'))
            .get('mina-toolbar .submenus a.active')
            .then((a: any) => expect(a.text().trim()).equals('memory'));
        }
      });
  });

  it('display root children in the table', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          expect(state.resource.children.length).above(0);
          expect(state.resource).to.equal(state.activeResource);
          cy.get('app-memory-resources-table .mina-table')
            .get('.row:not(.head)')
            .should('have.length.above', 0);
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
            .slice(0, Math.min(10, state.activeResource.children.length))
            .forEach((child: MemoryResource, i: number) => {
              cy.get(`app-memory-resources-table .row[idx="${i}"] > span:nth-child(4)`)
                .then((span: any) => expect(span.text().trim()).equals(transform(child.value)));
            });
        }
      });
  });

  it('display correct percentage', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          state.activeResource.children
            .slice(0, Math.min(10, state.activeResource.children.length))
            .forEach((child: MemoryResource, i: number) => {
              const expected = Math.round(child.value / state.activeResource.value * 1000) / 10;
              cy.get(`app-memory-resources-table .row[idx="${i}"] .perc`)
                .then((span: any) => expect(span.text().trim()).equals(`${expected}%`));
            });
        }
      });
  });

  it('display correct color based on percentage', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          state.activeResource.children
            .slice(0, Math.min(10, state.activeResource.children.length))
            .forEach((child: MemoryResource, i: number) => {
              const expected = Math.round(child.value / state.activeResource.value * 1000) / 10;
              if (expected > 50) {
                cy.get(`app-memory-resources-table .row[idx="${i}"] .perc`)
                  .then((span: any) => expect(span).to.have.class('red').and.not.have.class('yellow'))
                  .get(`app-memory-resources-table .row[idx="${i}"] .progress-background .progress-background`);
              } else if (expected > 10) {
                cy.get(`app-memory-resources-table .row[idx="${i}"] .perc`)
                  .then((span: any) => expect(span).to.have.class('yellow').and.not.have.class('red'))
                  .get(`app-memory-resources-table .row[idx="${i}"] .progress-background .progress-background`);
              } else {
                cy.get(`app-memory-resources-table .row[idx="${i}"] .perc`)
                  .then((span: any) => expect(span).to.not.have.class('red').and.not.have.class('yellow'))
                  .get(`app-memory-resources-table .row[idx="${i}"] .progress-background .progress-background`);
              }
            });
        }
      });
  });

  it('click on a row to show it\'s children', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length > 0);
          if (rowIndex >= 0) {
            const expectedActiveResourceName = state.activeResource.children[rowIndex].name.executableName;
            cy.get(`app-memory-resources-table .row[idx="${rowIndex}"]`)
              .click()
              .wait(500)
              .get(`app-memory-resources-table .row:not(.head)`)
              .each((row: any, i: number) => {
                expect(row.find('span:nth-child(3)').text().trim()).equals(state.activeResource.children[rowIndex].children[i].name.executableName);
                expect(row.find('span:nth-child(4)').text().trim()).equals(transform(state.activeResource.children[rowIndex].children[i].value));
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

  it('click on a row will show the children in the treemap', () => {
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
              .get('app-memory-resources-treemap .treemap g')
              .each((g: any, i: number) => {
                expect(g.find('text.name').text().trim()).equals(state.activeResource.children[rowIndex].children[i].name.executableName);
                expect(g.find('text.val').text().trim()).equals(transform(state.activeResource.children[rowIndex].children[i].value));
              });
          }
        }
      });
  });

  it('click on a row will add that item in the breadcrumbs', () => {
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
              .get('app-memory-resources nav span.breadcrumb')
              .should('have.length', 2)
              .get('app-memory-resources nav span.breadcrumb:last-child')
              .then((span: any) => expect(span.text().trim()).equals(state.activeResource.children[rowIndex].name.executableName));
          }
        }
      });
  });

  it('click on a row will show it as the active one', () => {
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
              .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(1) div')
              .then((span: any) => expect(span.text().trim()).equals(state.activeResource.children[rowIndex].name.executableName + ' ' + transform(state.activeResource.children[rowIndex].value)));
          }
        }
      });
  });

  it('click on a row without children will do nothing', () => {
    cy.window()
      .its('store')
      .then(getResources)
      .then((state: MemoryResourcesState) => {
        if (condition(state)) {
          const rowIndex = state.activeResource.children.findIndex(c => c.children.length === 0);
          if (rowIndex >= 0) {
            const expectedActiveResourceName = state.activeResource.name.executableName;
            cy.get(`app-memory-resources-table .row[idx="${rowIndex}"]`)
              .should('have.css', 'pointer-events', 'none')
              .click({ force: true })
              .wait(500)
              .get('app-memory-resources-table .row:not(.head)')
              .each((row: any, i: number) => {
                expect(row.find('span:nth-child(3)').text().trim()).equals(state.activeResource.children[i].name.executableName);
                expect(row.find('span:nth-child(4)').text().trim()).equals(transform(state.activeResource.children[i].value));
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
              .get(`app-memory-resources-table .row:not(.head)`)
              .each((row: any, i: number) => {
                expect(row.find('span:nth-child(3)').text().trim()).equals(state.activeResource.children[rowIndex].children[i].name.executableName);
                expect(row.find('span:nth-child(4)').text().trim()).equals(transform(state.activeResource.children[rowIndex].children[i].value));
              })
              .get('app-memory-resources-toolbar .fx-row-vert-cent .fx-row-vert-cent:nth-child(1) > span')
              .click()
              .wait(500)
              .get('app-memory-resources-table .row:not(.head)')
              .each((row: any, i: number) => {
                expect(row.find('span:nth-child(3)').text().trim()).equals(state.activeResource.children[i].name.executableName);
                expect(row.find('span:nth-child(4)').text().trim()).equals(transform(state.activeResource.children[i].value));
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
              .get(`app-memory-resources-table .row:not(.head)`)
              .each((row: any, i: number) => {
                expect(row.find('span:nth-child(3)').text().trim()).equals(state.activeResource.children[rowIndex].children[i].name.executableName);
                expect(row.find('span:nth-child(4)').text().trim()).equals(transform(state.activeResource.children[rowIndex].children[i].value));
              })
              .get('body')
              .type('{esc}')
              .wait(500)
              .get('app-memory-resources-table .row:not(.head)')
              .each((row: any, i: number) => {
                expect(row.find('span:nth-child(3)').text().trim()).equals(state.activeResource.children[i].name.executableName);
                expect(row.find('span:nth-child(4)').text().trim()).equals(transform(state.activeResource.children[i].value));
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
