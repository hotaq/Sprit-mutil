#!/bin/bash

# Sprite Tmux Profile 4: Three-Pane Layout
# Layout: Two agents on top row, one large agent on bottom
# Best for: 3 agents with prioritized main agent

SPRITE_SESSION="${SPRITE_SESSION:-sprite-session}"
AGENT_COUNT="${AGENT_COUNT:-3}"
MAIN_AGENT="${MAIN_AGENT:-1}"

echo "🖼️  Applying tmux Profile 4: Three-Pane Layout"

# Validate main agent
if [ "$MAIN_AGENT" -lt 1 ] || [ "$MAIN_AGENT" -gt "$AGENT_COUNT" ]; then
    echo "❌ Invalid main agent: $MAIN_AGENT (must be 1-$AGENT_COUNT)"
    exit 1
fi

# Kill existing session if it exists
tmux kill-session -t "$SPRITE_SESSION" 2>/dev/null || true

# Create new session
tmux new-session -d -s "$SPRITE_SESSION"

if [ "$AGENT_COUNT" -ge 1 ]; then
    # Set up main agent as large bottom pane
    tmux send-keys -t "$SPRITE_SESSION:0" "cd agents/$MAIN_AGENT && echo '🎯 Main Agent $MAIN_AGENT (Large View)'" C-m
    tmux rename-window -t "$SPRITE_SESSION:0" "main"

    if [ "$AGENT_COUNT" -ge 2 ]; then
        # Split horizontally for top row
        tmux split-window -v -t "$SPRITE_SESSION:main"

        # Find another agent for top-left
        local top_left_agent=1
        if [ "$top_left_agent" -eq "$MAIN_AGENT" ]; then
            top_left_agent=2
        fi

        tmux send-keys -t "$SPRITE_SESSION:main.1" "cd agents/$top_left_agent && echo '🤖 Agent $top_left_agent'" C-m

        if [ "$AGENT_COUNT" -ge 3 ]; then
            # Split top pane horizontally for third agent
            tmux split-window -h -t "$SPRITE_SESSION:main.1"

            # Find third agent
            local top_right_agent=1
            while [ "$top_right_agent" -eq "$MAIN_AGENT" ] || [ "$top_right_agent" -eq "$top_left_agent" ]; do
                top_right_agent=$((top_right_agent + 1))
            done

            tmux send-keys -t "$SPRITE_SESSION:main.2" "cd agents/$top_right_agent && echo '🤖 Agent $top_right_agent'" C-m
        fi
    fi

    # Create supervisor window
    tmux new-window -t "$SPRITE_SESSION" -n "supervisor"
    tmux send-keys -t "$SPRITE_SESSION:supervisor" "echo '🎮 Supervisor Panel' && echo 'Main focus: Agent $MAIN_AGENT'" C-m

    # Create additional windows for extra agents
    for i in $(seq 1 $AGENT_COUNT); do
        if [ "$i" -ne "$MAIN_AGENT" ] && [ "$i" -ne "$top_left_agent" ] && [ "$i" -ne "$top_right_agent" ] 2>/dev/null; then
            tmux new-window -t "$SPRITE_SESSION" -n "agent-$i"
            tmux send-keys -t "$SPRITE_SESSION:agent-$i" "cd agents/$i && echo '🤖 Agent $i'" C-m
        fi
    done
fi

# Set main-horizontal layout
tmux select-layout -t "$SPRITE_SESSION:main" main-horizontal

echo "✅ Profile 4 applied successfully!"
echo "🎯 Main agent: $MAIN_AGENT"
echo "🎮 Use 'sprite attach' to connect to the session"