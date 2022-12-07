import { join } from 'path';
import esbuild from 'esbuild';
import * as fs from 'fs/promises';
import glob from 'fast-glob';

const watch = process.argv.includes('--watch');
const minify = process.argv.includes('--minify');
console.log('Bundle options:', { watch, minify });

const bundleDest = minify ? 'bundle.min.js' : 'bundle.js';
const sourcemap = !minify;
const logLevel = 'info';

await esbuild.build({
    bundle: true,
    entryPoints: [join('web', 'index.tsx')],
    outfile: join('src', 'assets', bundleDest),
    platform: 'browser',
    watch,
    minify,
    sourcemap,
    logLevel,
    color: true,
});

await esbuild.build({
    entryPoints: [
        join('web', 'style.css'),
        join('node_modules', 'github-markdown-css', 'github-markdown.css'),
        ...(await glob(join('node_modules', 'highlight.js', 'styles', '*.css'))),
    ],
    outdir: join('src', 'assets'),
    platform: 'browser',
    watch,
    minify: true,
    sourcemap: false,
    logLevel,
    color: true,
});

await fs.copyFile(join('web', 'index.html'), join('src', 'assets', 'index.html'));
