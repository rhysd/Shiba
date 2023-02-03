import { join } from 'path';
import esbuild from 'esbuild';
import { copyFile } from 'fs/promises';
import glob from 'fast-glob';

const watch = process.argv.includes('--watch');
const minify = process.argv.includes('--minify');
console.log('Bundle options:', { watch, minify });

const bundleDest = minify ? 'bundle.min.js' : 'bundle.js';
const sourcemap = !minify;
const buildTsOptions = {
    bundle: true,
    entryPoints: [join('web', 'index.tsx')],
    outfile: join('src', 'assets', bundleDest),
    platform: 'browser',
    minify,
    sourcemap,
    logLevel: 'info',
    color: true,
};
const buildCssOptions = {
    entryPoints: [
        join('web', 'style.css'),
        join('node_modules', 'github-markdown-css', 'github-markdown.css'),
        ...(await glob('node_modules/highlight.js/styles/*.css')), // '/' is always a path separator for fast-glob
    ],
    outdir: join('src', 'assets'),
    platform: 'browser',
    minify: true,
    sourcemap: false,
    logLevel: 'info',
    color: true,
};

await copyFile(join('web', 'index.html'), join('src', 'assets', 'index.html'));

if (watch) {
    const tsCtx = await esbuild.context(buildTsOptions);
    await tsCtx.watch();
    const cssCtx = await esbuild.context(buildCssOptions);
    await cssCtx.watch();
} else {
    await esbuild.build(buildTsOptions);
    await esbuild.build(buildCssOptions);
}
