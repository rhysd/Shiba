import { join, dirname } from 'node:path';
import { copyFile, readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import process from 'node:process';
import esbuild from 'esbuild';
import { generateMathJaxLoader } from './mathjax.mjs';

if (process.argv.includes('--help')) {
    console.log(`node bundle.mjs [OPTIONS]

Options:

--watch    : Watch file changes and bundle changed files automatically
--minify   : Minify the bundled files
--metafile : Output the bundle metadata file as meta.json
--help     : Show this help
`);
    process.exit(0);
}

const absWorkingDir = dirname(dirname(fileURLToPath(import.meta.url)));
const hljsDefaultCssPlugin = {
    name: 'hljs-default-css',

    setup(build) {
        build.onEnd(async result => {
            if (result.errors.length > 0) {
                return;
            }
            const stylesDir = join(absWorkingDir, 'src', 'assets', 'node_modules', 'highlight.js', 'styles');
            const light = await readFile(join(stylesDir, 'github.css'), 'utf8');
            const dark = await readFile(join(stylesDir, 'github-dark.css'), 'utf8');
            const content = `@media(prefers-color-scheme:light){\n${light}}\n@media(prefers-color-scheme:dark){\n${dark}}\n`;
            const out = join(absWorkingDir, 'src', 'assets', 'ui', 'hljs_default.css');
            await writeFile(out, content, 'utf8');
            console.log('Generated ' + out);
        });
    },
};

console.log('Arguments:', process.argv);
const watch = process.argv.includes('--watch');
const minify = process.argv.includes('--minify');
const metafile = process.argv.includes('--metafile');
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
    alias: {
        // This alias redirects an import in @mathjax/src/mjs/output/svg/DefaultFont.js.
        // MathJax v4 adopts mathjax-newcm-font by default. Its SVG backend imports the font data statically so esbuild
        // always bundles the font data.
        //
        // https://github.com/mathjax/MathJax-src/blob/60ed165d2305b078e3d0fdaefa95366fde844cff/ts/output/svg/DefaultFont.ts#L3
        //
        // However we use mathjax-tex-font instead of the font because mathjax-newcm-font requires dynamic loading which
        // is not possible if bundling all JS sources. The mathjax-newcm-font package is actually unused in our use case.
        // This alias purges @mathjax/mathjax-tex-font dependency from @mathjax/src package and reduces the bundle size
        // by about 961KB.
        '#default-font/svg/default.js': '@mathjax/mathjax-tex-font/mjs/svg/default.js',
    },
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
    plugins: [hljsDefaultCssPlugin],
};

await copyFile(join(absWorkingDir, 'ui', 'index.html'), join(absWorkingDir, 'src', 'assets', 'index.html'));

const mathjaxLoaderPath = await generateMathJaxLoader();
console.log('Generated MathJax loader script:', mathjaxLoaderPath);

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
            'Meta file for the JS bundle was output to meta.json. Analyze it at https://esbuild.github.io/analyze/',
        );
    }
}
