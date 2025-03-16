import { defineConfig } from '@rsbuild/core';
import { pluginSolid } from '@rsbuild/plugin-solid';
import { pluginSass } from '@rsbuild/plugin-sass';
import { pluginBabel } from '@rsbuild/plugin-babel';
import { pluginWasmPack } from 'rsbuild-plugin-wasmpack';

export default defineConfig({
  html: {
    template: './index.html',
  },
  plugins: [
    pluginSolid(),
    pluginSass(),
    pluginBabel({
      include: /\.(?:jsx|tsx)$/,
    }),
    pluginWasmPack({
      crates: [
        {
          path: '../../amazegen',
          target: 'bundler',
        },
        {
          path: '../../amazegen_printer',
          target: 'bundler',
        },
      ],
      wasmpackPath: '/usr/bin/wasm-pack',
    }),
  ],
});
