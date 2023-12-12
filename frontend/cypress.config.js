const { defineConfig } = require('cypress')

module.exports = defineConfig({
  chromeWebSecurity: false,
  viewportWidth: 1920,
  viewportHeight: 1080,
  component: {
    devServer: {
      framework: 'angular',
      bundler: 'webpack',
    },
    specPattern: '**/*.cy.ts',
  },
  numTestsKeptInMemory: 20,
  experimentalMemoryManagement: true,
  defaultCommandTimeout: 10000,
  video: false,
  e2e: {
    baseUrl: 'http://frontend-server:80',
    setupNodeEvents(on, config) {
      return require('./cypress/plugins/index.js')(on, config);
      // return require('cypress-real-events/support')(on, config);
    }
  },
  include: [
    '**/*.ts',
    '**/*.js',
    'node_modules/rxjs/**/*.js'
  ]
});
