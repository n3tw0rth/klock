// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import catppuccin from "@catppuccin/starlight";

// https://astro.build/config
export default defineConfig({
  integrations: [
    starlight({
      title: 'Jir[ed]',
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/n3tw0rth/jired' }],
      sidebar: [
        {
          label: 'Guides',
          items: [
            { label: 'Installation', slug: 'guides/installation' },
            { label: 'Quick Start', slug: 'guides/quick' },
          ],
        },
        //{
        //  label: 'Reference',
        //  autogenerate: { directory: 'reference' },
        //},
      ],
      plugins: [
        catppuccin({
          dark: { flavor: "mocha", accent: "sapphire" },
          light: { flavor: "latte", accent: "blue" },
        }),
      ],
    }),
  ],
});
