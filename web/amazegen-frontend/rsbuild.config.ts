import { defineConfig } from '@rsbuild/core';
import { pluginSolid } from '@rsbuild/plugin-solid';
import { pluginSass } from '@rsbuild/plugin-sass';
import { pluginBabel } from '@rsbuild/plugin-babel';

export default defineConfig({
  plugins: [
    pluginSolid(),
    pluginSass(),
    pluginBabel({
      include: /\.(?:jsx|tsx)$/,
    }),
  ],
});
