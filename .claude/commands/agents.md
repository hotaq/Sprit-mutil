---
description: List and manage agents in the current sprite session.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Outline

Parse the user input to extract:
1. **subcommand** - List, show, create, remove, validate, provision
2. **agent_id** - Specific agent identifier (for show/create/remove)
3. **parameters** - Additional parameters for agent operations
4. **flags** - Optional flags like --force, --workspace, etc.

The user input format should be: `/agents [subcommand] [agent_id] [parameters...]`

## Workflow

1. **Subcommand routing**:
   - If no subcommand provided, default to listing agents
   - Route to appropriate agent operation based on subcommand

2. **List agents** (default or `/agents list`):
   - Load current agent configuration
   - Show all configured agents with their status
   - Display workspace paths and current activity
   - Show tmux session mapping

3. **Show agent** (`/agents show <id>`):
   - Display detailed information about specific agent
   - Show configuration, status, workspace details
   - Include last activity and resource usage

4. **Create agent** (`/agents create <id>`):
   - Create new agent configuration
   - Set up workspace and git worktree
   - Initialize agent settings and profiles
   - Validate agent creation

5. **Remove agent** (`/agents remove <id>`):
   - Validate agent exists and can be safely removed
   - Clean up agent workspace and configuration
   - Remove from active session if needed

6. **Validate agents** (`/agents validate`):
   - Check all agent configurations are valid
   - Verify workspaces and git setups
   - Report any issues or inconsistencies

7. **Provision agents** (`/agents provision [id]`):
   - Set up agent environments and dependencies
   - Initialize workspaces and configurations
   - Prepare agents for active use

## Examples

- `/agents` - List all agents with status
- `/agents list` - Same as above
- `/agents show 1` - Show detailed info about agent 1
- `/agents show frontend` - Show info about agent named 'frontend'
- `/agents create 4 --branch feature/new-agent` - Create new agent
- `/agents remove 3 --force` - Forcefully remove agent
- `/agents validate` - Check all agent configurations
- `/agents provision` - Set up all agent environments

## Notes

- Agent IDs can be numbers (1, 2, 3) or descriptive names
- Use list command to see current agent status and identify by ID
- Create operations may require additional parameters like branch or workspace
- Remove operations are destructive - use with caution
- Validation checks configuration and setup consistency
- Provisioning sets up runtime environments and dependencies
