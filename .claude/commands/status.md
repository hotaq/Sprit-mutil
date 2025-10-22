---
description: Display the current status of agents, sessions, and system health.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

Parse the user input to extract:
1. **scope** - overall, agents, session, system
2. **agent_id** - Specific agent for detailed status (optional)
3. **flags** - Optional flags like --detailed, --cleanup, --health

The user input format should be: `/status [scope] [agent_id] [flags...]`

## Workflow

1. **Determine status scope**:
   - If no scope provided, show overall status summary
   - Route to appropriate status reporting based on scope

2. **Overall status** (default or `/status overall`):
   - Show sprite session status
   - Display agent count and active status
   - Show system resource usage
   - Include any critical issues or warnings

3. **Agent status** (`/status agents` or `/status agents <id>`):
   - List all agents with current status
   - Show workspace activity and recent operations
   - Display agent health and responsiveness
   - For specific agent: detailed status with metrics

4. **Session status** (`/status session`):
   - Show tmux session information
   - Display session health and uptime
   - Show active windows and panes
   - Include connection status and performance

5. **System status** (`/status system`):
   - Display system resource usage
   - Show sprite runtime performance
   - Report any system issues or warnings
   - Include cleanup and optimization suggestions

6. **Health check** (`/status --health`):
   - Run comprehensive health assessment
   - Check agent connectivity and responsiveness
   - Validate workspace integrity and access
   - Report performance metrics and bottlenecks

7. **Cleanup operations** (`/status --cleanup`):
   - Clean up temporary files and old sessions
   - Optimize agent workspaces
   - Remove stale tmux sessions
   - Report cleanup results

## Examples

- `/status` - Overall status summary
- `/status agents` - Status of all agents
- `/status agents 1` - Detailed status of agent 1
- `/status session` - Tmux session information
- `/status system` - System resource status
- `/status --detailed` - Detailed status for all components
- `/status --health` - Run health check
- `/status --cleanup` - Clean up and report status

## Notes

- Overall status provides quick overview of system health
- Agent status includes activity, responsiveness, and workspace info
- Session status focuses on tmux and connection state
- System status covers resources and performance
- Use --detailed flag for comprehensive information
- Use --health for diagnostic health checks
- Use --cleanup to maintain system health and performance
- Status information is cached to reduce performance impact
