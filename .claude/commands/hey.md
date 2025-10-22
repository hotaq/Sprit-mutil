---
description: Send a command to a specific agent in the current tmux session.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

Parse the user input to extract:
1. **target_agent** - The name/identifier of the agent to send the command to
2. **command** - The command to execute (optional, defaults to opening interactive session)
3. **args** - Additional arguments for the command (optional)
4. **flags** - Optional flags like --timeout, --work-dir, --interactive

The user input format should be: `/hey <agent-name> [command] [args...] [flags]`

## Workflow

1. **Validate spritesession**: 
   - Check if sprite session exists and is active
   - Verify target agent is configured and running
   - Report any session or agent issues

2. **Parse command**:
   - Extract target agent (first argument after `/hey`)
   - Parse command and arguments 
   - Handle optional flags (--timeout, --work-dir, --interactive)
   - Set defaults (timeout: 60s, work-dir: current, interactive: false)

3. **Execute command**:
   - Load agent configuration from `agents/agents.yaml`
   - Find the agent's tmux pane in the current session
   - Set up environment variables and working directory
   - Send the command to the agent's pane
   - Handle interactive mode if requested

4. **Report results**:
   - Confirm command delivery to agent
   - Provide session and agent status
   - Show any errors or warnings
   - Attach to session if interactive mode

## Examples

- `/hey agent-1` - Open interactive session with agent-1
- `/hey agent-2 cargo test` - Run cargo test in agent-2's workspace
- `/hey frontend npm run build --work-dir ../frontend` - Build frontend project
- `/hey dev git status --timeout 30` - Check git status with 30s timeout
- `/hey python-agent python main.py --interactive` - Run Python script interactively

## Notes

- Agent names must match configured agents in `agents/agents.yaml`
- Commands execute in the agent's designated workspace directory
- Use `--interactive` flag to attach to the agent's tmux pane
- Multiple agents can receive commands independently
- Agent status and session health are verified before command execution
