# Implementation Plan: Multi-Agent System Slash Command Framework

**Branch**: `002-slash-command-framework` | **Date**: 2025-10-22 | **Spec**: spec.md
**Input**: Feature specification from `/specs/002-slash-command-framework/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Extend the existing speckit command framework with multi-agent communication capabilities, enabling agent-to-agent messaging via slash commands while maintaining framework-agnostic operation across Claude Code, Codex, Droid, and other AI assistants. The system will use a plugin-based adapter architecture with message queue communication and git-tracked context persistence.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.75+
**Primary Dependencies**: Enhanced stack including clap 4.4, tokio 1.35, tracing, figment, async-openai, message queue libraries (lapin, redis)
**Storage**: Git-tracked markdown files for context, YAML for agent configs, hybrid Unix socket + tokio channels for inter-agent communication
**Testing**: Comprehensive testing with assert_cmd, mockall, criterion, 80% coverage target using tarpaulin
**Target Platform**: Cross-platform CLI (Linux, macOS, Windows)
**Project Type**: Single CLI application with plugin architecture
**Performance Goals**: Simple commands <2s, complex operations <5s (95th percentile)
**Constraints**: Must integrate with existing speckit framework, maintain compatibility with Claude Code infrastructure
**Scale/Scope**: Support 3+ AI frameworks, handle 100+ concurrent commands, manage multiple agents

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Code Quality Compliance
- ✅ **Modularity**: Plugin-based adapter architecture ensures single-responsibility components
- ✅ **Documentation**: All public APIs and slash command interfaces will be documented with markdown
- ✅ **Style Guide**: Will follow Rust standard conventions (rustfmt, clippy)
- ⚠️ **Peer Review**: Requires at least one code review before merge (process to be established)

### Testing Standards Compliance
- ✅ **Unit Testing**: 80% code coverage target for all new components
- ✅ **Integration Testing**: Agent-to-agent communication workflows will be tested
- ✅ **E2E Testing**: Critical user journeys (slash command execution, template init) covered
- ✅ **Regression Prevention**: Bug fixes will include reproducing tests

### UX Consistency Compliance
- ✅ **Design System**: Follows existing speckit command patterns and Claude Code integration
- ✅ **Predictable Patterns**: Slash commands follow established `/speckit.*` conventions
- ⚠️ **Accessibility**: CLI accessibility to be evaluated (screen reader compatibility)
- ✅ **User Feedback**: Clear error messages and command status reporting

### Performance Requirements Compliance
- ✅ **Response Times**: <2s simple, <5s complex operations meets performance standards
- ✅ **Resource Efficiency**: Message queue and async runtime designed for efficiency
- ✅ **Scalability**: Plugin architecture supports horizontal scaling
- ⚠️ **Monitoring**: Performance monitoring implementation to be determined

**GATE STATUS**: ✅ PASSED - Minor clarifications needed for monitoring and CLI accessibility

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Revised Architecture: CLI-Based Agent Coordination

Based on user feedback, the architecture has been simplified to focus on **CLI-based agent coordination** rather than web APIs:

### Core Understanding
- **All agents are CLI processes** running in different tmux panels/sessions
- **Sprite CLI** coordinates between these local agents
- **Slash commands** are the interface for users to control the system
- **Communication** happens via tmux or direct CLI commands between agents
- **No APIs needed** because everything is local CLI-based

### System Architecture
```
┌─────────────────┐    Slash Commands    ┌─────────────────┐
│   User Terminal │ ──────────────────► │   Sprite CLI    │
└─────────────────┘                     └─────────────────┘
                                                │
                                                │ tmux commands
                                                ▼
┌─────────────────┐    tmux send-keys    ┌─────────────────┐
│ Agent Panel 1   │ ◄─────────────────── │   Sprite CLI    │
│ (claude-code)   │                     └─────────────────┘
└─────────────────┘
                                                │
                                                │ message passing
                                                ▼
┌─────────────────┐    tmux send-keys    ┌─────────────────┐
│ Agent Panel 2   │ ◄─────────────────── │   Sprite CLI    │
│ (another agent) │                     └─────────────────┘
└─────────────────┘
```

### Key Components (Simplified)
1. **Enhanced tmux communication** - Better message passing between tmux sessions
2. **Slash command parser** - Parse commands like `/hey agent message`
3. **Agent discovery** - Find which tmux session has which agent
4. **Message routing** - Route messages between tmux sessions
5. **Command execution** - Execute commands in specific agent panels

### What We DON'T Need (Removed from Scope)
- ❌ HTTP APIs
- ❌ External service calls
- ❌ Network communication
- ❌ Cloud integrations
- ❌ Complex message queues (Redis/RabbitMQ)

### CLAUDE.md Integration

The CLAUDE.md file must include examples of slash command usage for agent coordination:

```markdown
## Agent Coordination via Slash Commands

### Available Slash Commands
- `/hey <agent-name> <message>` - Send message to specific agent
- `/broadcast <message>` - Send message to all agents
- `/task <agent-name> <task-description>` - Assign task to agent
- `/review <agent-name> <content>` - Request code/content review
- `/context share <agent-name>` - Share current context

### Usage Examples

#### Feature Development Coordination
```bash
# Coordinate a new feature implementation
/hey orchestrator "I need to implement user authentication. Please coordinate the workflow."

# Expected response: orchestrator assigns tasks to specialized agents
# Check progress on each agent
/status coder
/status docs
/status tester

# Request review when complete
/review docs "Please review the authentication implementation"
```

#### Bug Fix Coordination
```bash
# Report bug and coordinate fix
/hey coder "Bug reported: Users can't login with expired tokens."
/hey tester "Please create reproduction case for expired token bug."

# After fix:
/hey tester "Please test the expired token fix"
/broadcast "Expired token bug has been fixed and tested."
```

### Agent Roles
- **orchestrator**: Coordinates workflows, distributes tasks
- **coder**: Implements features, fixes bugs, code reviews
- **docs**: Creates documentation, API specs
- **tester**: Writes tests, performs QA
```

## Complexity Tracking

*Fill ONLY if Constitution Check has violations that must be justified*

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |

