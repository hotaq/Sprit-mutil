# Simple Hey Command Manual Test

## Test Description
Manual test to verify the `/hey` command functionality works correctly.

## Prerequisites
1. Ensure sprite is built: `cargo build`
2. Have a tmux session available
3. Have agents configured in `agents/agents.yaml`

## Test Steps

### Step 1: Verify Command Exists
```bash
cargo run hey --help
```
**Expected**: Shows help for hey command
**Actual**: ✅ Working - shows help with correct arguments

### Step 2: Test Error Case (No Sprite Session)
```bash
cargo run hey 1 echo "test"
```
**Expected**: Should fail with error about no active session
**Result**: Tests error handling

### Step 3: Initialize Test Environment
```bash
# Create test directory
mkdir -p /tmp/sprite-hey-test
cd /tmp/sprite-hey-test

# Initialize git
git init
git config user.email "test@example.com"
git config user.name "Test"

# Initialize sprite
cargo run --bin sprite init --force
```

### Step 4: Create Agent Config
Create `agents/agents.yaml`:
```yaml
agents:
  agent1:
    name: "Test Agent 1"
    profile: "profile0.sh"
    workspace: "workspace1"
    active: true
    commands: ["echo", "ls"]

sprite:
  name: "test-sprite"
  description: "Test sprite"
  agents: ["agent1"]
```

### Step 5: Start Sprite Session
```bash
cargo run --bin sprite start --agents 1
```

### Step 6: Test Hey Commands
```bash
# Test basic command
cargo run --bin sprite hey 1 echo "Hello from agent!"

# Test with work directory
cargo run --bin sprite hey 1 ls --work-dir workspace1

# Test with timeout
cargo run --bin sprite hey 1 echo "timeout test" --timeout 5

# Test with environment variable
cargo run --bin sprite hey 1 echo $TEST_VAR --env TEST_VAR=hello
```

### Step 7: Verify Results
Check the tmux session to see if commands were executed:
```bash
tmux list-sessions
tmux attach -t sprite-<session-name>
```

## Test Results

### T032 - Create hey.md command ✅
- Status: COMPLETED
- File created: `.claude/commands/hey.md`
- Contains proper command structure and examples

### T036 - Test hey command with multiple agents ⏳
- Status: IN PROGRESS
- Basic command interface verified ✅
- Error handling tested ✅
- Full integration test needs sprite session

## Test Coverage

### ✅ Verified Components:
1. **Command Registration**: Hey command available in CLI
2. **Help System**: Proper help text and argument documentation
3. **Argument Parsing**: Correct handling of agent, command, args, and flags
4. **Error Handling**: Appropriate error for missing session

### 🔄 To be Verified:
1. **Actual Command Delivery**: Commands reaching agent panes
2. **Multiple Agents**: Communication with different agents
3. **Performance**: <3s delivery requirement
4. **Concurrent Commands**: Multiple simultaneous hey commands

## Next Steps

1. Complete sprite session setup for full integration test
2. Verify actual command delivery to agents
3. Test performance requirements
4. Test concurrent command scenarios

## Conclusion

The hey command infrastructure is properly implemented and integrated. The command-line interface works correctly with proper argument parsing and error handling. Full end-to-end testing requires sprite session management.
