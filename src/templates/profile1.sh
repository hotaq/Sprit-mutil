#!/bin/bash

# Sprite Tmux Profile 1: Left Column + Stacked Right
# Layout: Supervisor on left, agents stacked vertically on right
# Best for: 3-4 agents with detailed monitoring

SPRITE_SESSION="${SPRITE_SESSION:-sprite-session}"
AGENT_COUNT="${AGENT_COUNT:-3}"

echo "🖼️  Applying tmux Profile 1: Left Column + Stacked Right"

# Kill existing session if it exists
tmux kill-session -t "$SPRITE_SESSION" 2>/dev/null || true

# Create new session with supervisor window
tmux new-session -d -s "$SPRITE_SESSION" -n "supervisor"

# Enable mouse support for better navigation
tmux set-option -g mouse on
tmux set-option -t "$SPRITE_SESSION" mouse on

tmux send-keys -t "$SPRITE_SESSION:supervisor" "echo '🎯 Supervisor Control Panel'" C-m

if [ "$AGENT_COUNT" -ge 1 ]; then
    # Split to create right column
    tmux split-window -h -t "$SPRITE_SESSION:supervisor"

    # Put first agent in right pane
    tmux send-keys -t "$SPRITE_SESSION:supervisor.1" "cd agents/1 && echo '🤖 Agent 1 Workspace'" C-m

    # If more agents, split right pane vertically
    if [ "$AGENT_COUNT" -ge 2 ]; then
        tmux split-window -v -t "$SPRITE_SESSION:supervisor.1"
        tmux send-keys -t "$SPRITE_SESSION:supervisor.2" "cd agents/2 && echo '🤖 Agent 2 Workspace'" C-m
    fi

    if [ "$AGENT_COUNT" -ge 3 ]; then
        tmux split-window -v -t "$SPRITE_SESSION:supervisor.2"
        tmux send-keys -t "$SPRITE_SESSION:supervisor.3" "cd agents/3 && echo '🤖 Agent 3 Workspace'" C-m
    fi

    if [ "$AGENT_COUNT" -ge 4 ]; then
        tmux split-window -v -t "$SPRITE_SESSION:supervisor.3"
        tmux send-keys -t "$SPRITE_SESSION:supervisor.4" "cd agents/4 && echo '🤖 Agent 4 Workspace'" C-m
    fi

    # Create additional windows for extra agents
    for i in $(seq 5 $AGENT_COUNT); do
        tmux new-window -t "$SPRITE_SESSION" -n "agent-$i"
        tmux send-keys -t "$SPRITE_SESSION:agent-$i" "cd agents/$i && echo '🤖 Agent $i Workspace'" C-m
    done
fi

# Set main-vertical layout
tmux select-layout -t "$SPRITE_SESSION:supervisor" main-vertical

echo "✅ Profile 1 applied successfully!"
echo "🖱️  Mouse support enabled - Click to switch between panels"
echo "⌨️  Keyboard shortcuts: Ctrl+B then Arrow keys to navigate"
echo "🎮 Use 'sprite attach' to connect to the session"