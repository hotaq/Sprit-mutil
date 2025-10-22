---
description: Send a command to all active agents simultaneously.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

Parse the user input to extract:
1. **command** - The command to broadcast to all agents
2. **args** - Additional arguments for the command (optional)
3. **flags** - Optional flags like --timeout, --work-dir, --priority, --dry-run

The user input format should be: `/broadcast <command> [args...] [flags]`

## Workflow

1. **Validate sprite session**:
   - Check if sprite session exists and is active
   - Verify there are active agents
   - Report session and agent status

2. **Parse command**:
   - Extract target command (first argument after `/broadcast`)
   - Parse additional arguments
   - Handle optional flags (--timeout, --work-dir, --priority, --dry-run)
   - Set defaults (timeout: 30s, priority: normal, dry-run: false)

3. **Validate configuration**:
   - Check priority levels (low, normal, high, critical)
   - Validate work directory exists if specified
   - Parse environment variables if provided

4. **Execute broadcast**:
   - Execute the broadcast command using the send command infrastructure
   - Send command to all active agents in parallel
   - Track delivery status and responses
   - Handle dry-run mode if requested

5. **Report results**:
   - Show broadcast summary with number of agents
   - Display delivery status for each agent
   - Report any errors or failures
   - Show execution results or summary

## Examples

- `/broadcast cargo test` - Run tests on all agents
- `/broadcast echo "Hello everyone"` - Send message to all agents
- `/broadcast npm run build --priority high` - High priority broadcast
- `/broadcast docker-compose up --work-dir ../infrastructure --timeout 60` - Long-running command
- `/broadcast git status --dry-run` - Preview broadcast without execution

## Notes

- Broadcast commands execute in parallel by default for efficiency
- Use `--priority` to set message importance (low, normal, high, critical)
- Use `--dry-run` to preview what would be broadcast
- Use `--timeout` to set custom timeout for long operations
- Commands are sent to all active agents regardless of their current workspace
- Failed broadcasts are reported with individual agent status
