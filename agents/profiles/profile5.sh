#!/bin/bash

# Sprite Tmux Profile 5: Six-Pane Dashboard
# Layout: 2x3 grid of agent panes + supervisor window
# Best for: 6 agents with dashboard monitoring

SPRITE_SESSION="${SPRITE_SESSION:-sprite-session}"
AGENT_COUNT="${AGENT_COUNT:-6}"

echo "🖼️  Applying tmux Profile 5: Six-Pane Dashboard"

# Kill existing session if it exists
tmux kill-session -t "$SPRITE_SESSION" 2>/dev/null || true

# Create new session with first agent
tmux new-session -d -s "$SPRITE_SESSION"
tmux send-keys -t "$SPRITE_SESSION:0" "cd agents/1 && echo '🤖 Agent 1 Dashboard'" C-m
tmux rename-window -t "$SPRITE_SESSION:0" "dashboard"

# Create 2x3 grid
if [ "$AGENT_COUNT" -ge 2 ]; then
    tmux split-window -h -t "$SPRITE_SESSION:dashboard"
    tmux send-keys -t "$SPRITE_SESSION:dashboard.1" "cd agents/2 && echo '🤖 Agent 2 Dashboard'" C-m
fi

if [ "$AGENT_COUNT" -ge 3 ]; then
    tmux split-window -v -t "$SPRITE_SESSION:dashboard.0"
    tmux send-keys -t "$SPRITE_SESSION:dashboard.2" "cd agents/3 && echo '🤖 Agent 3 Dashboard'" C-m
fi

if [ "$AGENT_COUNT" -ge 4 ]; then
    tmux split-window -v -t "$SPRITE_SESSION:dashboard.1"
    tmux send-keys -t "$SPRITE_SESSION:dashboard.3" "cd agents/4 && echo '🤖 Agent 4 Dashboard'" C-m
fi

if [ "$AGENT_COUNT" -ge 5 ]; then
    tmux split-window -v -t "$SPRITE_SESSION:dashboard.2"
    tmux send-keys -t "$SPRITE_SESSION:dashboard.4" "cd agents/5 && echo '🤖 Agent 5 Dashboard'" C-m
fi

if [ "$AGENT_COUNT" -ge 6 ]; then
    tmux split-window -v -t "$SPRITE_SESSION:dashboard.3"
    tmux send-keys -t "$SPRITE_SESSION:dashboard.5" "cd agents/6 && echo '🤖 Agent 6 Dashboard'" C-m
fi

# Create supervisor window for monitoring
tmux new-window -t "$SPRITE_SESSION" -n "supervisor"
tmux send-keys -t "$SPRITE_SESSION:supervisor" "echo '📊 Dashboard Supervisor' && echo 'Monitoring $AGENT_COUNT agents'" C-m

# Set tiled layout for dashboard window
tmux select-layout -t "$SPRITE_SESSION:dashboard" tiled

# Create additional windows for extra agents
for i in $(seq 7 $AGENT_COUNT); do
    tmux new-window -t "$SPRITE_SESSION" -n "agent-$i"
    tmux send-keys -t "$SPRITE_SESSION:agent-$i" "cd agents/$i && echo '🤖 Agent $i'" C-m
done

echo "✅ Profile 5 applied successfully!"
echo "📊 Dashboard configured for $AGENT_COUNT agents"
echo "🎮 Use 'sprite attach' to connect to the session"
echo "💡 Dashboard window shows agents 1-6 in 2x3 grid"