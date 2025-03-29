import { join, dirname } from 'node:path';
import { copyFile, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import esbuild from 'esbuild';

if (process.argv.includes('--help')) {
    console.log(`node bundle.mjs [OPTIONS]

Options:

--watch    : Watch file changes and bundle changed files automatically
--minify   : Minify the bundled files
--metafile : Output the bundle metadata file as meta.json
`);
    process.exit(0);
}

console.log(process.argv);

const watch = process.argv.includes('--watch');
const minify = process.argv.includes('--minify');
const metafile = process.argv.includes('--metafile');
const absWorkingDir = dirname(dirname(fileURLToPath(import.meta.url)));
const {
    compilerOptions: { target },
} = JSON.parse(await readFile(join(absWorkingDir, 'tsconfig.json'), 'utf8'));
console.log('Bundle options:', { watch, minify, target, metafile });

const bundleDest = minify ? 'bundle.min.js' : 'bundle.js';
const sourcemap = !minify;
const buildTsOptions = {
    bundle: true,
    entryPoints: [join('ui', 'index.tsx')],
    outfile: join('src', 'assets', bundleDest),
    platform: 'browser',
    target,
    minify,
    sourcemap,
    logLevel: 'info',
    color: true,
    absWorkingDir,
    metafile,
};
const buildCssOptions = {
    entryPoints: [
        join('ui', 'style.css'),
        join('node_modules', 'github-markdown-css', 'github-markdown.css'),
        join('node_modules', 'highlight.js', 'styles', '*.css'),
    ],
    outdir: join('src', 'assets'),
    platform: 'browser',
    minify: true,
    sourcemap: false,
    logLevel: 'info',
    color: true,
    absWorkingDir,
};

await copyFile(join(absWorkingDir, 'ui', 'index.html'), join(absWorkingDir, 'src', 'assets', 'index.html'));

if (watch) {
    const tsCtx = await esbuild.context(buildTsOptions);
    await tsCtx.watch();
    const cssCtx = await esbuild.context(buildCssOptions);
    await cssCtx.watch();
} else {
    const [ts, _] = await Promise.all([esbuild.build(buildTsOptions), esbuild.build(buildCssOptions)]);
    if (metafile) {
        await writeFile('meta.json', JSON.stringify(ts.metafile));
        console.log(
            'Meta file for the JS bundle was output to meta.json. Analyze it at https://esbuild.github.io/analyze/'
        );
    }
}
