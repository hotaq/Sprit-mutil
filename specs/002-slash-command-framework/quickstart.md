# Quick Start Guide: Multi-Agent System Slash Command Framework

**Version**: 1.0.0
**Last Updated**: 2025-10-22

This guide will help you get started with the Multi-Agent System Slash Command Framework in under 15 minutes.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust 1.75+** with Cargo
- **Git** for version control
- **Tmux** for terminal multiplexing
- **A supported AI framework** (Claude Code, OpenAI API access, etc.)

### Installation Commands

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install tmux (macOS)
brew install tmux

# Install tmux (Ubuntu/Debian)
sudo apt-get install tmux

# Install tmux (Windows - via WSL)
# Follow Ubuntu/Debian instructions in WSL
```

## Step 1: Initialize Your Project

The `sprite init` command sets up a new multi-agent project with all necessary templates and configurations.

```bash
# Create a new project directory
mkdir my-multi-agent-project
cd my-multi-agent-project

# Initialize the project with sprite
sprite init

# Follow the interactive prompts:
# - Project name: my-multi-agent-project
# - Author: Your Name
# - Template: default (includes CLAUDE.md, SKILL.md, work_structure.md)
# - Frameworks: claude-code, openai (or your preferred frameworks)
```

### What Gets Created?

```
my-multi-agent-project/
├── CLAUDE.md              # Project-level agent instructions
├── SKILL.md              # Project capabilities and skills
├── work_structure.md     # Agent work organization rules
├── sprite.yaml           # Project configuration
├── agents/               # Agent-specific configurations
│   ├── orchestrator/
│   │   ├── CLAUDE.md
│   │   └── SKILL.md
│   ├── developer/
│   │   ├── CLAUDE.md
│   │   └── SKILL.md
│   └── documenter/
│       ├── CLAUDE.md
│       └── SKILL.md
├── .claude/
│   └── commands/         # Slash command definitions
└── .gitignore           # Git ignore file
```

## Step 2: Configure Your First Agent

Edit the agent configuration to match your needs.

```bash
# Edit the orchestrator agent configuration
vim agents/orchestrator/CLAUDE.md
```

### Sample Agent Configuration

```markdown
# Orchestrator Agent

You are the orchestrator agent for this multi-agent system. Your role is to:

1. Coordinate between different agents
2. Manage task distribution
3. Monitor overall system health
4. Handle user requests and route them appropriately

## Capabilities
- Task coordination and distribution
- System monitoring and health checks
- User request routing
- Agent communication management

## Communication
- Use the message queue for agent-to-agent communication
- Respond to user requests within 2 seconds
- Provide clear status updates

## Framework Integration
- Primary framework: claude-code
- Backup framework: openai-gpt4
```

## Step 3: Configure API Keys

Set up your AI framework API keys.

```bash
# Edit the project configuration
vim sprite.yaml
```

### Sample Configuration

```yaml
project:
  name: "my-multi-agent-project"
  version: "1.0.0"
  author: "Your Name"

frameworks:
  claude-code:
    api_key: "${CLAUDE_API_KEY}"
    base_url: "https://api.anthropic.com"
    model: "claude-3-sonnet-20240229"

  openai:
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"
    model: "gpt-4"

communication:
  method: "message_queue"
  timeout: 30
  retry:
    max_attempts: 3
    initial_delay: 1000
    backoff_multiplier: 2.0

agents:
  - name: "orchestrator"
    type: "orchestrator"
    framework: "claude-code"
    capabilities: ["coordination", "monitoring", "routing"]

  - name: "developer"
    type: "developer"
    framework: "claude-code"
    capabilities: ["coding", "testing", "debugging"]

  - name: "documenter"
    type: "documenter"
    framework: "openai"
    capabilities: ["documentation", "analysis", "reporting"]
```

### Set Environment Variables

```bash
# Set your API keys as environment variables
export CLAUDE_API_KEY="your-claude-api-key-here"
export OPENAI_API_KEY="your-openai-api-key-here"

# Add to your shell profile for persistence
echo 'export CLAUDE_API_KEY="your-claude-api-key-here"' >> ~/.zshrc
echo 'export OPENAI_API_KEY="your-openai-api-key-here"' >> ~/.zshrc
```

## Step 4: Create Your First Slash Command

Create a custom slash command to test the system.

```bash
# Create a new slash command
vim .claude/commands/hello.md
```

### Sample Slash Command

```markdown
# hello

**Description**: Send a greeting message to a specific agent

**Usage**: `/hello <agent-name> [message]`

**Parameters**:
- `agent-name`: Name of the target agent (required)
- `message`: Custom message (optional, defaults to "Hello!")

**Examples**:
- `/hello orchestrator`
- `/hello developer "Can you help me with this code?"`

**Category**: communication

**Handler**: agent_communication

**Framework Compatibility**: claude-code, openai
```

## Step 5: Test the System

Start the sprite system and test your first command.

```bash
# Start the sprite system
sprite start

# In a new terminal, test your command
sprite hello orchestrator

# Expected output:
# ✅ Message sent to orchestrator: "Hello!"
# 📬 Response from orchestrator: "Hello! I'm ready to help coordinate tasks."
```

## Step 6: Create a Shortcut

Define a shortcut for frequently used command sequences.

```bash
# Create a shortcut for agent status check
sprite shortcut create status "agents list --format table" --description "Check all agent statuses"

# Use the shortcut
sprite status

# Expected output:
# ┌─────────────┬────────┬─────────────┬─────────────┐
# │ Agent       │ Status │ Last Seen   │ Messages    │
# ├─────────────┼────────┼─────────────┼─────────────┤
# │ orchestrator│ Active │ 2 seconds   │ 1,234       │
# │ developer   │ Idle   │ 5 seconds   │ 892         │
# │ documenter  │ Busy   │ 1 second    │ 456         │
# └─────────────┴────────┴─────────────┴─────────────┘
```

## Step 7: Multi-Agent Communication

Test agent-to-agent communication.

```bash
# Send a task to the developer agent via the orchestrator
sprite hey orchestrator "Ask the developer to create a new Rust function for data validation"

# Send a broadcast message to all agents
sprite broadcast "System maintenance scheduled for 2:00 AM UTC"

# Direct communication with a specific agent
sprite hey developer "Please review the validation logic in src/validation.rs"
```

## Common Workflows

### Workflow 1: Code Development with Multiple Agents

```bash
# 1. Start a new feature development task
sprite hey orchestrator "Start new feature: user authentication system"

# 2. Orchestrator coordinates with developer
sprite hey developer "Create user authentication module with JWT support"

# 3. Developer implements and responds
# (Developer works on the code...)

# 4. Request documentation
sprite hey documenter "Create documentation for the new authentication system"

# 5. Review the complete work
sprite agents list --status completed
```

### Workflow 2: System Health Monitoring

```bash
# Check overall system status
sprite status

# Monitor specific agent
sprite agent monitor orchestrator

# Check message queue status
sprite queue status

# View recent agent communications
sprite messages recent --limit 10
```

### Workflow 3: Template-based Project Setup

```bash
# List available templates
sprite templates list

# Create a new microservice project
sprite template init microservice-rust my-new-service

# Configure the new service's agents
cd my-new-service
vim sprite.yaml

# Start the new service's agent system
sprite start
```

## Configuration Options

### Performance Tuning

```yaml
# sprite.yaml
performance:
  # Response time targets
  simple_command_timeout: 2  # seconds
  complex_command_timeout: 5  # seconds

  # Concurrency limits
  max_concurrent_commands: 100
  max_concurrent_agents: 10

  # Resource limits
  memory_limit_mb: 512
  cpu_limit_percent: 80
```

### Communication Settings

```yaml
# sprite.yaml
communication:
  # Message queue settings
  queue_size: 1000
  batch_size: 10

  # Retry configuration
  retry:
    max_attempts: 3
    initial_delay_ms: 1000
    max_delay_ms: 10000
    backoff_multiplier: 2.0

  # Performance monitoring
  metrics_enabled: true
  tracing_enabled: true
```

### Agent-specific Overrides

```yaml
# sprite.yaml
agent_overrides:
  developer:
    timeout: 10  # Longer timeout for complex coding tasks
    retry:
      max_attempts: 5  # More retries for development work

  documenter:
    framework: "openai"  # Use different framework
    model: "gpt-4-turbo"  # Specific model choice
```

## Troubleshooting

### Common Issues

**Issue**: Commands timeout after 2 seconds
```bash
# Check system status
sprite status

# Increase timeout for complex commands
sprite config set performance.complex_command_timeout 10
```

**Issue**: Agent not responding
```bash
# Check agent health
sprite agent health developer

# Restart specific agent
sprite agent restart developer

# Check agent logs
sprite logs developer --tail 50
```

**Issue**: API key authentication errors
```bash
# Verify API keys are set
echo $CLAUDE_API_KEY
echo $OPENAI_API_KEY

# Test API connectivity
sprite test api claude-code
sprite test api openai
```

**Issue**: Message queue full
```bash
# Check queue status
sprite queue status

# Clear old messages
sprite queue cleanup --older-than 1h

# Increase queue size
sprite config set communication.queue_size 2000
```

### Getting Help

```bash
# Get help with any command
sprite help <command>

# List all available commands
sprite commands list

# Show system information for debugging
sprite info --verbose

# Check for known issues
sprite doctor
```

## Next Steps

Now that you have a basic multi-agent system running, consider these next steps:

1. **Add More Agents**: Create specialized agents for specific tasks
2. **Custom Commands**: Develop custom slash commands for your workflows
3. **Integration**: Integrate with external tools and APIs
4. **Monitoring**: Set up comprehensive monitoring and alerting
5. **Templates**: Create custom templates for different project types

### Advanced Topics

- **Custom Framework Adapters**: Build adapters for other AI frameworks
- **Message Routing**: Implement advanced message routing logic
- **Performance Optimization**: Tune system performance for your use case
- **Security**: Implement authentication and authorization
- **Scaling**: Scale to handle larger workloads

## Resources

- **Documentation**: [Full Documentation](https://docs.sprite.dev)
- **GitHub Repository**: [sprite-mutilagent](https://github.com/sprite/sprite-mutilagent)
- **Community**: [Discord Server](https://discord.gg/sprite)
- **Examples**: [Example Projects](https://github.com/sprite/examples)

## Support

If you encounter any issues or have questions:

1. Check the [troubleshooting guide](troubleshooting.md)
2. Search [existing issues](https://github.com/sprite/sprite-mutilagent/issues)
3. Create a [new issue](https://github.com/sprite/sprite-mutilagent/issues/new)
4. Join our [Discord community](https://discord.gg/sprite)

---

**Congratulations! 🎉** You now have a working multi-agent system with slash command support. Start building amazing things with your AI agent team!