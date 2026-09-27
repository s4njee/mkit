// mdBook preprocessor: show the exact workspace GPUI pin on every chapter.
import fs from 'node:fs';
import path from 'node:path';

function pinnedVersion(bookRoot) {
    const manifest = fs.readFileSync(path.join(bookRoot, '..', 'Cargo.toml'), 'utf8');
    let inDependencies = false;
    let foundSection = false;
    let line;
    for (const part of manifest.split(/\r?\n/)) {
        const heading = part.match(/^\s*\[([^\]]+)\]\s*$/);
        if (heading) {
            inDependencies = heading[1] === 'workspace.dependencies';
            foundSection ||= inDependencies;
        } else if (inDependencies && /^\s*gpui_pre\s*=/.test(part)) {
            line = part;
            break;
        }
    }
    if (!foundSection) throw new Error('Cargo.toml has no [workspace.dependencies] section');
    if (!line) throw new Error('workspace.dependencies has no gpui_pre pin');
    const table = line.match(/^\s*gpui_pre\s*=\s*\{([^}]*)\}/);
    if (!table || !/\bpackage\s*=\s*"gpui-pre"/.test(table[1])) {
        throw new Error('workspace gpui_pre dependency must use gpui-pre');
    }
    const version = table[1].match(/\bversion\s*=\s*"=(\d+\.\d+\.\d+)"/);
    if (!version) throw new Error('workspace gpui-pre version must be pinned exactly');
    return version[1];
}

function addBanner(sections, banner) {
    for (const section of sections) {
        const chapter = section.Chapter;
        if (!chapter) continue;
        chapter.content = banner + chapter.content;
        addBanner(chapter.sub_items || [], banner);
    }
}

const args = process.argv[1] === 'supports' ? process.argv.slice(1) : process.argv.slice(2);
if (args[0] === 'supports') process.exit(args[1] === 'html' ? 0 : 1);
if (args.length) throw new Error('unsupported mdBook preprocessor command');

let input = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => { input += chunk; });
process.stdin.on('end', () => {
    const [context, book] = JSON.parse(input);
    const version = pinnedVersion(context.root);
    const banner = '<div class="gpui-version-banner"><span>Workspace GPUI pin:</span> '
        + `<code>gpui-pre ${version}</code></div>\n\n`;
    addBanner(book.sections || book.items, banner);
    process.stdout.write(JSON.stringify(book));
});
