#!/bin/bash

# Sprite Tmux Profile 3: Single Full-Width Top (Focus Mode)
# Layout: One agent in focus, supervisor as small overlay
# Best for: Deep focus on single agent work

SPRITE_SESSION="${SPRITE_SESSION:-sprite-session}"
AGENT_COUNT="${AGENT_COUNT:-1}"
FOCUS_AGENT="${FOCUS_AGENT:-1}"

echo "🖼️  Applying tmux Profile 3: Focus Mode - Agent $FOCUS_AGENT"

# Validate focus agent
if [ "$FOCUS_AGENT" -lt 1 ] || [ "$FOCUS_AGENT" -gt "$AGENT_COUNT" ]; then
    echo "❌ Invalid focus agent: $FOCUS_AGENT (must be 1-$AGENT_COUNT)"
    exit 1
fi

# Kill existing session if it exists
tmux kill-session -t "$SPRITE_SESSION" 2>/dev/null || true

# Create new session with focus agent
tmux new-session -d -s "$SPRITE_SESSION" -n "agent-$FOCUS_AGENT"

# Enable mouse support for better navigation
tmux set-option -g mouse on
tmux set-option -t "$SPRITE_SESSION" mouse on
tmux send-keys -t "$SPRITE_SESSION:agent-$FOCUS_AGENT" "cd agents/$FOCUS_AGENT && echo '🎯 Focus Mode: Agent $FOCUS_AGENT'" C-m

# Create small supervisor window
tmux new-window -t "$SPRITE_SESSION" -n "supervisor"
tmux send-keys -t "$SPRITE_SESSION:supervisor" "echo '🎮 Supervisor (Mini Panel)' && echo 'Current focus: Agent $FOCUS_AGENT'" C-m

# Create quick access windows for other agents
for i in $(seq 1 $AGENT_COUNT); do
    if [ "$i" -ne "$FOCUS_AGENT" ]; then
        tmux new-window -t "$SPRITE_SESSION" -n "agent-$i"
        tmux send-keys -t "$SPRITE_SESSION:agent-$i" "cd agents/$i && echo '🤖 Agent $i (Standby)'" C-m
    fi
done

# Switch to focus agent
tmux select-window -t "$SPRITE_SESSION:agent-$FOCUS_AGENT"

echo "✅ profile3 applied successfully!"
echo "🖱️  Mouse support enabled - Click to switch between panels"
echo "⌨️  Keyboard shortcuts: Ctrl+B then Arrow keys to navigate"
echo "🎯 Focus mode activated on Agent $FOCUS_AGENT"
echo "🎮 Use 'sprite attach' to connect to the session"
echo "💡 Use Ctrl+B then W to quickly switch between agents"