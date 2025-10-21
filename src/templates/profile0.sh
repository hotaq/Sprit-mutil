#!/bin/bash

# Sprite Tmux Profile 0: Supervisor + Agent Layout
# Layout:
#   - 1 agent: Supervisor top, agent bottom (vertical split)
#   - 2 agents: Supervisor left, agent right (horizontal split)
#   - 3+ agents: Supervisor top, 2 agents split below, extra agents in new windows
# Best for: 1-3 agents with active supervision

SPRITE_SESSION="${SPRITE_SESSION:-sprite-session}"
AGENT_COUNT="${AGENT_COUNT:-3}"

echo "🖼️  Applying tmux Profile 0: Top + Split Bottom"

# Kill existing session if it exists
tmux kill-session -t "$SPRITE_SESSION" 2>/dev/null || true

# Create new session with supervisor window
tmux new-session -d -s "$SPRITE_SESSION" -n "supervisor"

# Enable mouse support for better navigation
tmux set-option -g mouse on

# Enable mouse support for current session
tmux set-option -t "$SPRITE_SESSION" mouse on

tmux send-keys -t "$SPRITE_SESSION:supervisor" "echo '🎯 Supervisor Control Panel'" C-m

if [ "$AGENT_COUNT" -eq 1 ]; then
    # Single agent - split vertically (supervisor on top, agent below)
    tmux split-window -v -t "$SPRITE_SESSION:supervisor"
    tmux send-keys -t "$SPRITE_SESSION:supervisor.1" "cd agents/1 && echo '🤖 Agent 1 Workspace'" C-m

elif [ "$AGENT_COUNT" -eq 2 ]; then
    # Two agents - split horizontally from supervisor pane
    tmux split-window -h -t "$SPRITE_SESSION:supervisor"

    tmux send-keys -t "$SPRITE_SESSION:supervisor.0" "cd agents/1 && echo '🤖 Agent 1 Workspace'" C-m
    tmux send-keys -t "$SPRITE_SESSION:supervisor.1" "cd agents/2 && echo '🤖 Agent 2 Workspace'" C-m

else
    # 3+ agents - create main horizontal split, then split bottom
    tmux split-window -v -t "$SPRITE_SESSION:supervisor"
    tmux split-window -h -t "$SPRITE_SESSION:supervisor.1"

    tmux send-keys -t "$SPRITE_SESSION:supervisor.0" "cd agents/1 && echo '🤖 Agent 1 Workspace'" C-m
    tmux send-keys -t "$SPRITE_SESSION:supervisor.1" "cd agents/2 && echo '🤖 Agent 2 Workspace'" C-m
    tmux send-keys -t "$SPRITE_SESSION:supervisor.2" "cd agents/3 && echo '🤖 Agent 3 Workspace'" C-m

    # Create additional windows for extra agents (only if > 3)
    if [ "$AGENT_COUNT" -gt 3 ]; then
        for i in $(seq 4 $AGENT_COUNT); do
            tmux new-window -t "$SPRITE_SESSION" -n "agent-$i"
            tmux send-keys -t "$SPRITE_SESSION:agent-$i" "cd agents/$i && echo '🤖 Agent $i Workspace'" C-m
        done
    fi
fi

# Set balanced layout
tmux select-layout -t "$SPRITE_SESSION:supervisor" main-horizontal

echo "✅ Profile 0 applied successfully!"
echo "🖱️  Mouse support enabled - Click to switch between panels"
echo "⌨️  Keyboard shortcuts: Ctrl+B then Arrow keys to navigate"
echo "🎮 Use 'sprite attach' to connect to the session"