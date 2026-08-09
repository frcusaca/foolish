import { tool } from '@opencode-ai/plugin';
import { execFileSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import fs from 'node:fs';

// Resolve the ast-grep binary. Note: the legacy `sg` alias is deprecated upstream AND
// collides with /usr/bin/sg (the setgid command) on most Linux distros, so always
// prefer the explicit `ast-grep` name.
function astGrepBin() {
  const cargoBin = path.join(process.env.HOME || '/home/hcbusy', '.cargo', 'bin', 'ast-grep');
  return fs.existsSync(cargoBin) ? cargoBin : 'ast-grep';
}

const astGrepEngine = tool({
  description:
    'High-performance Abstract Syntax Tree query engine. Supports structural pattern discovery, rewrite refactoring, and composite node scanning.',
  args: {
    action: tool.schema
      .enum(['search', 'rewrite', 'scan'])
      .describe(
        'Operation mode: "search" reads matches; "rewrite" modifies source files; "scan" uses a custom inline YAML rule block.',
      ),
    lang: tool.schema
      .string()
      .describe('Target language profile (e.g., typescript, python, rust, go, javascript, cpp, java).'),
    pattern: tool.schema
      .string()
      .optional()
      .describe('The structural code template using metavariables. (Required if action is search or rewrite).'),
    rewrite: tool.schema
      .string()
      .optional()
      .describe('The replacement syntax structure utilizing bound metavariables. (Required if action is rewrite).'),
    yamlRule: tool.schema
      .string()
      .optional()
      .describe('A complete, multi-line ast-grep rule definition in YAML string format. (Required if action is scan).'),
    scope: tool.schema
      .string()
      .optional()
      .describe('Optional sub-path within the project to restrict the query to (e.g. a single crate directory).'),
  },
  async execute({ action, lang, pattern, rewrite, yamlRule, scope }, context) {
    // Establish sandboxed execution directory boundaries
    const projectRoot = context.directory ?? process.cwd();
    const target = scope ? path.resolve(projectRoot, scope) : projectRoot;

    if (!target.startsWith(projectRoot)) {
      return JSON.stringify({ success: false, error: 'scope must stay within the project directory.' }, null, 2);
    }

    const cargoBin = path.join(process.env.HOME || '/home/hcbusy', '.cargo', 'bin');
    const env = { ...process.env, PATH: `${cargoBin}:${process.env.PATH || '/usr/bin:/bin'}` };

    let tempRuleFile = null;
    // Build an argv array rather than a shell string: patterns routinely contain quotes,
    // $metavariables and backticks, all of which a shell would mangle or expand.
    let args;

    try {
      if (action === 'search') {
        if (!pattern) {
          return JSON.stringify({ success: false, error: 'Missing required string parameter: pattern' }, null, 2);
        }
        args = ['run', '--pattern', pattern, '--lang', lang];
      } else if (action === 'rewrite') {
        if (!pattern || !rewrite) {
          return JSON.stringify(
            { success: false, error: "Rewrite actions require both 'pattern' and 'rewrite' configurations." },
            null,
            2,
          );
        }
        args = ['run', '--pattern', pattern, '--rewrite', rewrite, '--update-all', '--lang', lang];
      } else {
        if (!yamlRule) {
          return JSON.stringify(
            { success: false, error: "Scan action requires a complete 'yamlRule' configuration parameter." },
            null,
            2,
          );
        }
        // Generate an ephemeral rule file in the system temp dir, never inside the
        // repository, so a crashed run cannot leave artifacts in the user's tree.
        tempRuleFile = path.join(os.tmpdir(), `ast_grep_rule_${process.pid}_${Date.now()}.yaml`);
        fs.writeFileSync(tempRuleFile, yamlRule, 'utf-8');
        args = ['scan', '--rule', tempRuleFile];
      }

      // Append project workspace scope target
      args.push(target);

      const stdout = execFileSync(astGrepBin(), args, {
        env,
        cwd: projectRoot,
        encoding: 'utf-8',
        stdio: ['ignore', 'pipe', 'pipe'],
        maxBuffer: 64 * 1024 * 1024,
      });

      return JSON.stringify(
        {
          success: true,
          action_executed: action,
          scope: path.relative(projectRoot, target) || '.',
          results:
            stdout.trim() ||
            'Operation completed successfully (No matches found or modifications applied safely).',
        },
        null,
        2,
      );
    } catch (error) {
      // ast-grep exits non-zero for "no matches" in some modes; surface stdout too so a
      // clean empty result is not reported as a hard failure.
      const stderr = error.stderr ? error.stderr.toString().trim() : '';
      const stdout = error.stdout ? error.stdout.toString().trim() : '';
      if (!stderr && !stdout) {
        return JSON.stringify(
          { success: true, action_executed: action, results: 'No matches found.' },
          null,
          2,
        );
      }
      return JSON.stringify({ success: false, error: stderr || stdout || error.message }, null, 2);
    } finally {
      // Always garbage collect ephemeral rule files to prevent pollution
      if (tempRuleFile && fs.existsSync(tempRuleFile)) {
        try {
          fs.unlinkSync(tempRuleFile);
        } catch {}
      }
    }
  },
});

export default astGrepEngine;

// ---------------------------------------------------------------------------
// CLI entrypoint. opencode imports this file (import.meta.main false, nothing
// below runs); a shell can execute it instead:
//
//   bun ast_grep_engine.js --action search --lang rust \
//       --pattern '$X.unwrap()' --scope foolish-core
//   bun ast_grep_engine.js --action scan --lang rust \
//       --yaml-rule-file rule.yaml --scope foolish-core
//
// --yaml-rule-file is offered alongside --yaml-rule because rule documents are
// multi-line YAML, which is awkward to pass as a single shell argument.
// ---------------------------------------------------------------------------
if (import.meta.main) {
  const argv = process.argv.slice(2);
  const has = (name) => argv.includes(name);
  const str = (name) => {
    const i = argv.indexOf(name);
    return i >= 0 ? argv[i + 1] : undefined;
  };

  if (has('--help') || has('-h') || argv.length === 0) {
    console.log(`ast_grep_engine — structural code search and rewrite

  bun ast_grep_engine.js --action <search|rewrite|scan> --lang <lang> [options]

  --action ACTION        search (read-only) | rewrite (mutates files) | scan (YAML rule)
  --lang LANG            rust, typescript, python, go, javascript, tsx, c, cpp, java, ruby, php
  --pattern PATTERN      structural template; required for search and rewrite
  --rewrite TEMPLATE     replacement template; required for rewrite
  --yaml-rule TEXT       inline ast-grep rule document; required for scan
  --yaml-rule-file PATH  read the rule document from a file instead
  --scope SUBPATH        restrict to a sub-path, e.g. a single crate directory
  -h, --help             this message

Metavariables are written $X / $$$ARGS, never backslash-escaped.
Runs against the current working directory.`);
    process.exit(0);
  }

  const ruleFile = str('--yaml-rule-file');
  const result = await astGrepEngine.execute(
    {
      action: str('--action'),
      lang: str('--lang'),
      pattern: str('--pattern'),
      rewrite: str('--rewrite'),
      yamlRule: ruleFile ? fs.readFileSync(ruleFile, 'utf-8') : str('--yaml-rule'),
      scope: str('--scope'),
    },
    {
      sessionID: 'cli',
      messageID: 'cli',
      agent: 'cli',
      directory: process.cwd(),
      worktree: process.cwd(),
      abort: new AbortController().signal,
      metadata() {},
      async ask() {},
    },
  );

  const parsed = JSON.parse(result);
  // Print the human-readable payload, not the JSON envelope the model consumes.
  console.log(parsed.success ? parsed.results : `error: ${parsed.error}`);
  process.exit(parsed.success ? 0 : 1);
}
