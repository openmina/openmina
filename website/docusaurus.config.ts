import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Mina Rust Node Documentation',
  tagline: 'Rust implementation of the Mina Protocol (originally OCaml) - lightweight blockchain using zero knowledge proofs',
  favicon: 'img/favicon.ico',

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: 'https://o1-labs.github.io',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/mina-rust/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'o1-labs', // Usually your GitHub org/user name.
  projectName: 'mina-rust', // Usually your repo name.

  onBrokenLinks: 'throw', // Throw error on broken links to enforce link integrity
  onBrokenAnchors: 'warn',

  // Markdown configuration
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },

  // Static directories for assets
  staticDirectories: ['static'],

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // Enable versioning
          includeCurrentVersion: true,
          lastVersion: 'current',
          versions: {
            current: {
              label: 'develop',
            },
          },
          // Enable edit links to GitHub
          editUrl: 'https://github.com/o1-labs/mina-rust/tree/develop/website/docs/',
        },
        blog: false, // Disable blog for technical documentation
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themes: ['docusaurus-theme-github-codeblock'],


  themeConfig: {
    // Replace with your project's social card
    image: 'img/rust-node-social-card.svg',
    // Default to dark mode
    colorMode: {
      defaultMode: 'dark',
      disableSwitch: false,
      respectPrefersColorScheme: false,
    },

    // SEO improvements and metadata
    metadata: [
      {name: 'keywords', content: 'Rust node, Mina Protocol, Rust, blockchain, zero knowledge proofs, zkProofs, cryptocurrency, decentralized'},
      {name: 'description', content: 'The Mina Rust Node is a Rust implementation of the Mina Protocol (originally written in OCaml) - a lightweight blockchain using zero knowledge proofs. Learn how to run nodes, develop applications, and understand the protocol.'},
      {name: 'author', content: 'o1Labs'},
      {name: 'robots', content: 'index,follow'},
      {name: 'googlebot', content: 'index,follow'},
      {property: 'og:type', content: 'website'},
      {property: 'og:title', content: 'Mina Rust Node Documentation'},
      {property: 'og:description', content: 'The Mina Rust Node is a Rust implementation of the Mina Protocol (originally written in OCaml) - a lightweight blockchain using zero knowledge proofs.'},
      {property: 'og:image', content: 'https://o1-labs.github.io/mina-rust/img/rust-node-social-card.svg'},
      {property: 'twitter:card', content: 'summary_large_image'},
      {property: 'twitter:title', content: 'Mina Rust Node Documentation'},
      {property: 'twitter:description', content: 'The Mina Rust Node is a Rust implementation of the Mina Protocol (originally written in OCaml) - a lightweight blockchain using zero knowledge proofs.'},
      {property: 'twitter:image', content: 'https://o1-labs.github.io/mina-rust/img/rust-node-social-card.svg'},
    ],

    // Algolia search configuration
    // Note: These credentials are safe to commit publicly.
    // The apiKey is search-only with read permissions.
    algolia: {
      appId: '5PSH5CKRTB',
      apiKey: '7da6823405ea3b6e55b9d6d8fef3526a',
      indexName: 'mina-rust',
      contextualSearch: true,
      searchParameters: {},
      searchPagePath: 'search',
    },

    navbar: {
      title: '',
      logo: {
        alt: 'Mina Rust Node Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'nodeRunnersSidebar',
          position: 'left',
          label: 'Node Operators',
        },
        {
          type: 'docSidebar',
          sidebarId: 'developersSidebar',
          position: 'left',
          label: 'Developers',
        },
        {
          type: 'docSidebar',
          sidebarId: 'researchersSidebar',
          position: 'left',
          label: 'Researchers',
        },
        {
          type: 'docSidebar',
          sidebarId: 'appendixSidebar',
          position: 'left',
          label: 'Appendix',
        },
        {
          href: 'https://o1-labs.github.io/mina-rust/api-docs/',
          label: 'API Docs',
          position: 'left',
        },
        {
          type: 'docsVersionDropdown',
          position: 'right',
          dropdownActiveClassDisabled: true,
        },
        {
          href: 'https://github.com/o1-labs/mina-rust',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Documentation',
          items: [
            {
              label: 'Node Operators',
              to: '/docs/node-operators/getting-started',
            },
            {
              label: 'Developers',
              to: '/docs/developers/architecture',
            },
            {
              label: 'Researchers',
              to: '/docs/researchers/protocol',
            },
            {
              label: 'Appendix',
              to: '/docs/appendix/docker-installation',
            },
            {
              label: 'API Documentation',
              href: 'https://o1-labs.github.io/mina-rust/api-docs/',
            },
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/o1-labs/mina-rust',
            },
            {
              label: 'Issues',
              href: 'https://github.com/o1-labs/mina-rust/issues',
            },
            {
              label: 'Discussions',
              href: 'https://github.com/o1-labs/mina-rust/discussions',
            },
            {
              label: 'Twitter',
              href: 'https://x.com/o1_labs',
            },
            {
              label: 'Discord',
              href: 'https://discord.gg/minaprotocol',
            },
            {
              label: 'Forums',
              href: 'https://forums.minaprotocol.com/',
            },
            {
              label: 'LinkedIn',
              href: 'https://www.linkedin.com/company/o1labs/',
            },
          ],
        },
        {
          title: 'Resources',
          items: [
            {
              label: 'Mina Protocol',
              href: 'https://minaprotocol.com/',
            },
            {
              label: 'Mina Protocol Docs',
              href: 'https://docs.minaprotocol.com/',
            },
            {
              label: 'o1Labs',
              href: 'https://www.o1labs.org/',
            },
            {
              label: 'o1Labs GitHub',
              href: 'https://github.com/o1-labs',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} o1Labs.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'toml', 'bash', 'docker', 'yaml', 'json'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
