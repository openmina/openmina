/**
 * This configuration is used for the staging-webnode environment.
 *
 * NOTE: When modifying environment configuration files, update the documentation at:
 * website/docs/developers/frontend/environment-configuration.mdx
 */

export default {
  production: true,
  canAddNodes: false,
  showWebNodeLandingPage: true,
  globalConfig: {
    features: {
      dashboard: [],
      'block-production': ['won-slots'],
    },
  },
  sentry: {
    dsn: 'https://69aba72a6290383494290cf285ab13b3@o4508216158584832.ingest.de.sentry.io/4508216160616528',
    tracingOrigins: ['https://www.openmina.com'],
  },
  configs: [
    {
      name: 'Web Node',
      isWebNode: true,
    },
  ],
};
