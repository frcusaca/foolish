# CCW Maven Setup Skill

## Purpose
Prepares the Maven build environment for Claude Code Web (CCW) by installing Java 25 and configuring the Maven proxy. Does nothing in local environments.

## When to Use
Run this skill automatically before any Maven build operations when working in Claude Code Web.

## What It Does
1. Detects if running in CCW environment (checks `CLAUDECODE` env var)
2. If in CCW:
   - Installs SDKMAN (if not present)
   - Installs latest stable Java 25 (Temurin) via SDKMAN
   - Starts local Maven authentication proxy
   - Configures Maven settings.xml
3. If not in CCW:
   - Does nothing (local environment already has Java 25)

## Usage
```bash
/skill ccw-maven-setup
```

Or directly:
```bash
prep_if_ccw.sh
```
NB: That script prep_if_ccw.sh is in the directory of this skill next to the skill.md and it may requires execution using bash due to lack of executable permissions.
```sh
bash prep_if_ccw.sh
```

## Technical Details
The local proxy is necessary because Maven doesn't honor standard HTTP_PROXY environment variables for authentication. The proxy runs on localhost:3128 and forwards authenticated requests to the CCW upstream proxy.

## References
- GitHub Issue: https://github.com/anthropics/claude-code/issues/13372
- LinkedIn Article: https://www.linkedin.com/pulse/fixing-maven-build-issues-claude-code-web-ccw-tarun-lalwani-8n7oc
