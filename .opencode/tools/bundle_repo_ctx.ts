import { tool } from '@opencode-ai/plugin';
import { execFileSync } from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';
/*
  This code can benefit from additional instructions in AGENTS.md:
"""
# OpenCode Local Execution Rules

## Tool Selection & VRAM Conservation Priorities
### Use the `bundle_repo_ctx` tool:
  * Immediately on your first turn of the session, with no arguments, to list the workspace crates.
  * Then call it again with `crate=<name>` to pull the full context for the one crate you are working on.
  * When restarting a session, to rescan for all file system and program source changes.
  * For all high-level codebase analysis, structural changes, cross-module debugging sessions, or project-wide refactoring planning to establish full view of code status.
  * Command line exploration is last resort, use only when the bundle_repo_ctx fails. Avoid `find`, `grep`, `ls` when authoritative information is available from `bundle_repo_ctx` tool.
  * Treat the output payload of `bundle_repo_ctx` as your single source of truth for repository structure and context mapping.
"""
*/

// Core hardcoded systemic blocks. `docs`, `.omo` and `tmp` are excluded by design:
// they are bulk prose/session state, not source, and on this workspace they alone
// accounted for several MB of payload.
const IGNORE_DIRS = new Set([
  '.git', 'node_modules', '__pycache__', 'dist', 'build', '.venv', 'venv',
  'target', 'docs', '.omo', 'tmp',
]);
const ALLOWED_EXTENSIONS = new Set([
  '.py', '.js', '.ts', '.tsx', '.json', '.jsonc', '.go', '.rs',
  '.cpp', '.h', '.cs', '.java', '.html', '.css', '.md', '.foo', '.einmo',
]);

// Verified einmo fixtures live under a `verified/` directory at arbitrary depth
// (e.g. einmo_suite/verified/foop/41/*.einmo). They are settled golden data that
// rarely informs a code question, and they are roughly a third of all .einmo files,
// so they are skipped unless `--do-not-skill-einmo-verified` is set.
const VERIFIED_SEGMENT = 'verified';
function isVerifiedEinmo(relPathPosix: string): boolean {
  if (!relPathPosix.endsWith('.einmo')) return false;
  // Only directory segments count — a file merely named "verified.einmo" is kept.
  return relPathPosix.split('/').slice(0, -1).includes(VERIFIED_SEGMENT);
}

const CARGO_BIN = path.join(process.env.HOME || '/home/hcbusy', '.cargo', 'bin');
const EXEC_ENV = { ...process.env, PATH: `${CARGO_BIN}:${process.env.PATH || '/usr/bin:/bin'}` };

type Crate = { name: string; dir: string };

function humanBytes(n: number): string {
  if (n < 1024) return `${n}B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)}K`;
  return `${(n / 1024 / 1024).toFixed(1)}M`;
}

// Helper: extract a human-readable rwx string from a numeric mode
function getPosixPermissions(mode: number): string {
  const isDir = (mode & fs.constants.S_IFMT) === fs.constants.S_IFDIR;
  const fType = isDir ? 'd' : '-';

  const rwxStr = [
    (mode & fs.constants.S_IRUSR) ? 'r' : '-',
    (mode & fs.constants.S_IWUSR) ? 'w' : '-',
    (mode & fs.constants.S_IXUSR) ? 'x' : '-',
    (mode & fs.constants.S_IRGRP) ? 'r' : '-',
    (mode & fs.constants.S_IWGRP) ? 'w' : '-',
    (mode & fs.constants.S_IXGRP) ? 'x' : '-',
    (mode & fs.constants.S_IROTH) ? 'r' : '-',
    (mode & fs.constants.S_IWOTH) ? 'w' : '-',
    (mode & fs.constants.S_IXOTH) ? 'x' : '-',
  ].join('');

  return fType + rwxStr;
}

function findGitRoot(dir: string): string | null {
  try {
    return execFileSync('git', ['rev-parse', '--show-toplevel'], {
      cwd: dir, encoding: 'utf-8', stdio: ['ignore', 'pipe', 'ignore'],
    }).trim() || null;
  } catch {
    return null;
  }
}

// Build the whole repo's git status in ONE subprocess instead of one per file.
function buildGitStatusMap(gitRoot: string): Map<string, string> | null {
  let raw: string;
  try {
    raw = execFileSync('git', ['status', '--porcelain', '--untracked-files=all', '-z'], {
      cwd: gitRoot, encoding: 'utf-8', stdio: ['ignore', 'pipe', 'ignore'],
      maxBuffer: 32 * 1024 * 1024,
    });
  } catch {
    return null;
  }

  const map = new Map<string, string>();
  // -z format: "XY <path>\0", and for renames/copies "XY <new>\0<old>\0"
  const fields = raw.split('\0');
  for (let i = 0; i < fields.length; i++) {
    const entry = fields[i];
    if (!entry) continue;
    const code = entry.slice(0, 2);
    map.set(entry.slice(3), code.trim() || '??');
    if (code[0] === 'R' || code[0] === 'C') i++;
  }
  return map;
}

// Walk upward for the Cargo.toml that declares [workspace].
function findWorkspaceRoot(startDir: string): string | null {
  let dir = startDir;
  for (;;) {
    const manifest = path.join(dir, 'Cargo.toml');
    try {
      if (fs.existsSync(manifest) && /^\s*\[workspace\]/m.test(fs.readFileSync(manifest, 'utf-8'))) {
        return dir;
      }
    } catch {
      // unreadable manifest — keep climbing
    }
    const parent = path.dirname(dir);
    if (parent === dir) return null;
    dir = parent;
  }
}

// Enumerate workspace members. `cargo metadata` is authoritative — it resolves glob
// members, exclusions and any package-name/directory-name mismatch. Fall back to a
// literal manifest parse when cargo is unavailable or the workspace cannot resolve.
function discoverCrates(workspaceRoot: string): Crate[] {
  try {
    const raw = execFileSync(
      'cargo',
      ['metadata', '--no-deps', '--offline', '--format-version', '1'],
      { cwd: workspaceRoot, env: EXEC_ENV, encoding: 'utf-8', stdio: ['ignore', 'pipe', 'ignore'], maxBuffer: 64 * 1024 * 1024 },
    );
    const meta = JSON.parse(raw);
    const crates: Crate[] = (meta.packages ?? [])
      .filter((p: any) => p.manifest_path)
      .map((p: any) => ({ name: p.name, dir: path.dirname(p.manifest_path) }))
      .filter((c: Crate) => c.dir.startsWith(workspaceRoot));
    if (crates.length) return crates.sort((a, b) => a.name.localeCompare(b.name));
  } catch {
    // fall through to the manifest parse
  }

  // Fallback: read `members = [...]` out of the workspace manifest directly.
  const crates: Crate[] = [];
  try {
    const manifest = fs.readFileSync(path.join(workspaceRoot, 'Cargo.toml'), 'utf-8');
    const block = manifest.match(/^\s*members\s*=\s*\[([\s\S]*?)\]/m);
    if (!block) return [];
    const patterns = [...block[1].matchAll(/["']([^"']+)["']/g)].map((m) => m[1]);

    const dirs: string[] = [];
    for (const p of patterns) {
      if (p.includes('*')) {
        // Expand a single trailing wildcard segment, e.g. "crates/*"
        const parent = path.join(workspaceRoot, path.dirname(p));
        try {
          for (const e of fs.readdirSync(parent, { withFileTypes: true })) {
            if (e.isDirectory()) dirs.push(path.join(parent, e.name));
          }
        } catch {}
      } else {
        dirs.push(path.join(workspaceRoot, p));
      }
    }

    for (const dir of dirs) {
      try {
        const m = fs.readFileSync(path.join(dir, 'Cargo.toml'), 'utf-8');
        const name = m.match(/^\s*name\s*=\s*["']([^"']+)["']/m)?.[1];
        if (name) crates.push({ name, dir });
      } catch {}
    }
  } catch {}
  return crates.sort((a, b) => a.name.localeCompare(b.name));
}

export default tool({
  description:
    'Gathers valid source code files for ONE cargo crate, maps its directory tree, and bundles them into a clean payload with permissions, timestamps and Git status, honoring exclusions from .claudeignore. Call with no crate to list the available crates in the workspace.',
  args: {
    crate: tool.schema
      .string()
      .optional()
      .describe('Name of the single workspace crate to bundle. Omit to list the available crates.'),
    maxDepth: tool.schema
      .number().int().min(0).default(8)
      .describe('Maximum depth to traverse the directory tree, relative to the crate root'),
    maxFileBytes: tool.schema
      .number().int().min(0).default(200_000)
      .describe('Skip the body of any single file larger than this many bytes'),
    '--do-not-skill-einmo-verified': tool.schema
      .boolean().default(false)
      .describe(
        'Include the verified einmo fixtures. By default any file matching **/verified/**/*.einmo is skipped, ' +
        'which removes about a third of the einmo test data. Set true to bundle them anyway.',
      ),
  },
  async execute(
    { crate, maxDepth, maxFileBytes, '--do-not-skill-einmo-verified': includeVerifiedEinmo },
    context,
  ) {
    // Prefer the session directory over process.cwd() — see ToolContext docs
    const cwd = context.directory ?? process.cwd();
    const workspaceRoot = findWorkspaceRoot(cwd);
    const root = workspaceRoot ?? cwd;
    const gitRoot = findGitRoot(root);
    const gitStatusMap = gitRoot ? buildGitStatusMap(gitRoot) : null;

    // --- .claudeignore, parsed relative to the workspace root -------------------
    const dynamicIgnoreRegexes: RegExp[] = [];
    const claudeIgnorePath = path.join(root, '.claudeignore');
    if (fs.existsSync(claudeIgnorePath)) {
      for (const line of fs.readFileSync(claudeIgnorePath, 'utf-8').split(/\r?\n/)) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith('#')) continue;

        const isDirRule = trimmed.endsWith('/');
        const bare = isDirRule ? trimmed.slice(0, -1) : trimmed;

        let regexStr = bare
          .replace(/[.+^${}()|[\]\\]/g, '\\$&') // Escape regex punctuation
          .replace(/\*/g, '[^/]*')              // Turn * into "anything but a separator"
          .replace(/\?/g, '[^/]');              // Turn ? into a single non-separator

        // Anchor the rule so `test` cannot swallow `latest.py`. A rule without a
        // slash matches at any depth; a rule with one is rooted at the repo top.
        regexStr = bare.includes('/') ? `^${regexStr}` : `(^|/)${regexStr}`;
        regexStr += isDirRule ? '(/|$)' : '$';

        try {
          dynamicIgnoreRegexes.push(new RegExp(regexStr));
        } catch {
          // Fallback if a messy line creates broken regex shapes
        }
      }
    }

    const toPosix = (p: string) => p.split(path.sep).join('/');
    const isIgnoredByClaude = (relativePath: string) =>
      dynamicIgnoreRegexes.some((re) => re.test(toPosix(relativePath)));

    function getGitStatus(fullPath: string): string {
      if (!gitStatusMap || !gitRoot) return '[Git: UNKNOWN]';
      const code = gitStatusMap.get(toPosix(path.relative(gitRoot, fullPath)));
      return code ? `[Git: ${code}]` : '[Git: Clean]';
    }

    // --- shared traversal -------------------------------------------------------
    type Walk = {
      fileTree: string[];
      sourceContent: string[];
      skipped: string[];
      fileCount: number;
      byteCount: number;
      verifiedEinmoSkipped: number;
    };

    function walk(dir: string, depth: number, out: Walk, withBodies: boolean) {
      if (depth > maxDepth) return;

      let entries: fs.Dirent[];
      try {
        entries = fs.readdirSync(dir, { withFileTypes: true });
      } catch (e) {
        out.skipped.push(`${path.relative(root, dir) || '.'} (unreadable: ${e})`);
        return;
      }

      for (const entry of entries) {
        if (IGNORE_DIRS.has(entry.name)) continue;

        const fullPath = path.join(dir, entry.name);
        const relPath = path.relative(root, fullPath);
        if (isIgnoredByClaude(relPath)) continue;

        // lstat, not stat: never follow symlinks, which would risk infinite recursion
        let stat: fs.Stats;
        try {
          stat = fs.lstatSync(fullPath);
        } catch {
          continue;
        }
        if (stat.isSymbolicLink()) continue;

        if (stat.isDirectory()) {
          if (withBodies) out.fileTree.push(`${'  '.repeat(depth)}📁 ${entry.name}/`);
          walk(fullPath, depth + 1, out, withBodies);
          continue;
        }
        if (!stat.isFile()) continue;
        if (!ALLOWED_EXTENSIONS.has(path.extname(entry.name))) continue;

        // Counted, not listed: 174 individual skip lines would drown the payload.
        if (!includeVerifiedEinmo && isVerifiedEinmo(toPosix(relPath))) {
          out.verifiedEinmoSkipped++;
          continue;
        }

        out.fileCount++;
        out.byteCount += stat.size;
        if (!withBodies) continue;

        const perms = getPosixPermissions(stat.mode);
        const lastModified = stat.mtime.toISOString().replace('T', ' ').replace('Z', '');
        const created = stat.birthtime.toISOString().replace('T', ' ').replace('Z', '');
        const gitStatus = getGitStatus(fullPath);

        out.fileTree.push(
          `${'  '.repeat(depth)}📄 [${perms}] ${gitStatus} ${entry.name} (Mod: ${lastModified})`,
        );

        if (stat.size > maxFileBytes) {
          out.skipped.push(`${relPath} (${stat.size} bytes > maxFileBytes ${maxFileBytes})`);
          continue;
        }

        try {
          const content = fs.readFileSync(fullPath, 'utf-8');
          out.sourceContent.push(
            `### FILE: ${toPosix(relPath)}\n` +
              `- **Perms:** \`${perms}\`\n` +
              `- **Created:** ${created}\n` +
              `- **Modified:** ${lastModified}\n` +
              `- **Git:** ${gitStatus}\n` +
              `\`\`\`${path.extname(entry.name).slice(1)}\n${content}\n\`\`\`\n`,
          );
        } catch (e) {
          out.sourceContent.push(`### FILE: ${toPosix(relPath)}\n// Error reading file context: ${e}\n`);
        }
      }
    }

    const newWalk = (): Walk =>
      ({ fileTree: [], sourceContent: [], skipped: [], fileCount: 0, byteCount: 0, verifiedEinmoSkipped: 0 });

    // One shared note so the model knows data was withheld and how to get it back.
    const verifiedNote = (n: number) =>
      n === 0
        ? ''
        : `\nVerified einmo fixtures skipped: ${n} (matched **/verified/**/*.einmo). ` +
          `Pass "--do-not-skill-einmo-verified": true to include them.`;

    // --- crate selection --------------------------------------------------------
    const crates = workspaceRoot ? discoverCrates(workspaceRoot) : [];

    let listingVerifiedSkipped = 0;
    function crateListing(): string {
      listingVerifiedSkipped = 0;
      return crates
        .map((c) => {
          const m = newWalk();
          walk(c.dir, 0, m, false);
          listingVerifiedSkipped += m.verifiedEinmoSkipped;
          const rel = toPosix(path.relative(root, c.dir)) || '.';
          return `- ${c.name.padEnd(20)} ${String(m.fileCount).padStart(4)} files  ${humanBytes(m.byteCount).padStart(6)}  (${rel}/)`;
        })
        .join('\n');
    }

    if (!workspaceRoot) {
      return {
        title: 'No cargo workspace found',
        output:
          `# NO CARGO WORKSPACE\nNo Cargo.toml declaring [workspace] was found at or above:\n  ${cwd}\n\n` +
          `This tool bundles one crate at a time and needs a cargo workspace to resolve crate names.`,
        metadata: { workspace: false },
      };
    }

    if (!crate) {
      const listing = crateListing() || '(none resolved)';
      return {
        title: `${crates.length} crate${crates.length === 1 ? '' : 's'} in ${path.basename(root)}`,
        output:
          `# WORKSPACE: ${path.basename(root)}\n` +
          `Root: ${root}\n` +
          `No crate selected. Call again with crate=<name> to bundle exactly one crate.\n` +
          `${verifiedNote(listingVerifiedSkipped)}\n\n` +
          `## AVAILABLE CRATES\n${listing}\n`,
        metadata: {
          workspace: true,
          crates: crates.map((c) => c.name),
          verifiedEinmoSkipped: listingVerifiedSkipped,
        },
      };
    }

    const selected = crates.find((c) => c.name === crate);
    if (!selected) {
      return {
        title: `Unknown crate: ${crate}`,
        output:
          `# UNKNOWN CRATE: ${crate}\nThat crate is not a member of this workspace.\n\n` +
          `## AVAILABLE CRATES\n${crateListing() || '(none resolved)'}\n`,
        metadata: { workspace: true, error: 'unknown_crate', crates: crates.map((c) => c.name) },
      };
    }

    // --- bundle the one crate ---------------------------------------------------
    const out = newWalk();
    walk(selected.dir, 0, out, true);

    // Workspace manifest travels with every crate bundle: it carries the shared
    // dependency versions and lints the crate's own manifest inherits from.
    let workspaceContext = '';
    try {
      const wsManifest = path.join(root, 'Cargo.toml');
      workspaceContext =
        `## WORKSPACE CONTEXT\n### FILE: Cargo.toml\n` +
        `- **Git:** ${getGitStatus(wsManifest)}\n` +
        `\`\`\`toml\n${fs.readFileSync(wsManifest, 'utf-8')}\n\`\`\`\n`;
    } catch (e) {
      workspaceContext = `## WORKSPACE CONTEXT\n// Error reading workspace Cargo.toml: ${e}\n`;
    }

    const output = `
# SYSTEM CONTEXT: CRATE SNAPSHOT
Scoped to a single crate. The payload below covers the workspace manifest, then this crate's file and directory structure with metadata, then source file content including permissions, creation and modification times, and Git status.

Workspace: ${root}
Crate:     ${selected.name}  (${toPosix(path.relative(root, selected.dir)) || '.'}/)
Sibling crates (not included): ${crates.filter((c) => c.name !== selected.name).map((c) => c.name).join(', ') || '(none)'}${verifiedNote(out.verifiedEinmoSkipped)}${gitStatusMap ? '' : '\n(Not a git work tree — all Git statuses reported as UNKNOWN.)'}

${workspaceContext}
## CRATE FILE TREE
\`\`\`text
${out.fileTree.join('\n')}
\`\`\`

## SOURCE CODE FILES
${out.sourceContent.join('\n')}
${out.skipped.length ? `\n## SKIPPED\n${out.skipped.map((s) => `- ${s}`).join('\n')}\n` : ''}`;

    return {
      title: `${selected.name}: ${out.fileCount} file${out.fileCount === 1 ? '' : 's'}, ${humanBytes(out.byteCount)}`,
      output,
      metadata: {
        workspace: true,
        crate: selected.name,
        fileCount: out.fileCount,
        byteCount: out.byteCount,
        skipped: out.skipped.length,
        verifiedEinmoSkipped: out.verifiedEinmoSkipped,
        git: Boolean(gitStatusMap),
      },
    };
  },
});
