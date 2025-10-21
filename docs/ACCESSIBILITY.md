# Accessibility Features for Sprite Multi-Agent Toolkit

This document outlines the accessibility features implemented in the Sprite multi-agent toolkit to ensure WCAG 2.1 AA compliance and make the tool usable by people with disabilities.

## Overview

Sprite is committed to providing an accessible command-line interface that can be used by everyone, including users with visual impairments, motor disabilities, and other accessibility needs.

## WCAG 2.1 AA Compliance

### Perceivable

#### 1.1.1 Non-text Content
- All emoji icons have text alternatives
- Screen reader mode replaces emojis with descriptive text
- High contrast mode available for better visibility

#### 1.3.3 Sensory Characteristics
- Information is not conveyed through color alone
- Text annotations and descriptions provided for all visual elements
- Multiple ways to identify status and actions

### Operable

#### 2.1.1 Keyboard
- Full keyboard navigation support
- No time-based interactions without user control
- All commands accessible via standard CLI interface

#### 2.4.3 Focus Order
- Consistent command structure and parameter ordering
- Predictable help system layout
- Clear hierarchy in information display

### Understandable

#### 3.1.1 Language of Page
- Clear, simple language in all output
- Consistent terminology throughout the application
- Avoids jargon where possible

#### 3.2.4 Consistent Identification
- Consistent use of icons and symbols
- Standardized error message format
- Predictable command patterns

### Robust

#### 4.1.1 Parsing
- Valid, well-structured output formatting
- Compatible with assistive technologies
- Follows CLI accessibility best practices

## Environment Variables

### NO_COLOR Support
Follows the [NO_COLOR specification](https://no-color.org/):

```bash
# Disable color output
export NO_COLOR=1
sprite hey 1 "command"
```

### Screen Reader Detection
Automatically detects common screen reader environments:

```bash
# Enable screen reader mode
export SCREEN_READER=1
sprite status
```

Supported screen reader indicators:
- `SCREEN_READER`
- `SR`
- `JAWS`
- `NVDA`
- `VOICEOVER`
- `TALKBACK`

### General Accessibility Mode

```bash
# Enable accessibility features
export ACCESSIBILITY=1
sprite send "command"
```

### Traditional Color Control

```bash
# Traditional method to disable colors
export CLICOLOR=0
sprite sync
```

## Features

### Screen Reader Support

#### Semantic Annotations
- Emojis replaced with descriptive text:
  - `✅` → `[SUCCESS]`
  - `❌` → `[ERROR]`
  - `⚠️` → `[WARNING]`
  - `ℹ️` → `[INFO]`
  - `📡` → `[BROADCAST]`
  - `🔄` → `[SYNC]`

#### Enhanced Context
- Additional descriptive text for complex operations
- Clear progress indicators with percentage and description
- Structured information presentation

#### Error Messages
- Enhanced error descriptions with actionable suggestions
- Context-aware help recommendations
- Clear next steps for resolution

### Visual Accessibility

#### High Contrast Mode
- Emoji replacement with text equivalents
- Clear separation between information types
- Enhanced readability options

#### Color Independence
- Information never conveyed through color alone
- Multiple visual indicators for status
- Text-based alternatives for all visual cues

### Motor Accessibility

#### Keyboard Operation
- Full keyboard navigation
- No mouse-dependent interactions
- Consistent command patterns

#### Time Management
- No time-limited interactions
- User-controlled pacing
- Clear indication of long-running operations

## Usage Examples

### Standard Usage
```bash
# Normal operation with visual indicators
sprite status
# Output: ✅ Session 'sprite-123' is healthy and running normally.
```

### Screen Reader Mode
```bash
# Enable screen reader compatibility
SCREEN_READER=1 sprite status
# Output: SUCCESS: Session 'sprite-123' is healthy and running normally.
```

### High Contrast Mode
```bash
# Disable colors and use text indicators
NO_COLOR=1 sprite send "command"
# Output: [BROADCAST] Broadcasting command to 3 agents: command
```

### Accessible Error Messages
```bash
# Enhanced error with suggestions
sprite hey 99 "command"
# Output: ERROR: Agent '99' not found.
#         Suggestion: Check if the agent exists and is properly configured.
```

## Testing

### Accessibility Test Suite
Run the accessibility test suite:

```bash
./test_accessibility.sh
```

### Manual Testing Checklist

#### Screen Reader Testing
- [ ] Test with `SCREEN_READER=1`
- [ ] Verify emoji-to-text conversion
- [ ] Check semantic annotations
- [ ] Validate enhanced descriptions

#### Visual Testing
- [ ] Test with `NO_COLOR=1`
- [ ] Verify color independence
- [ ] Check high contrast mode
- [ ] Validate text alternatives

#### Motor Testing
- [ ] Test keyboard-only operation
- [ ] Verify consistent command structure
- [ ] Check for time-limited interactions
- [ ] Validate error recovery

## Command Accessibility

### All Commands Support:
- `--help` with clear descriptions
- Consistent parameter ordering
- Progress indicators for long operations
- Clear success/error messaging
- Context-aware suggestions

### Specific Features:
- `sprite status` - Detailed session health reporting
- `sprite hey` - Individual agent communication with feedback
- `sprite send` - Broadcast with result aggregation
- `sprite sync` - Progress tracking and conflict guidance
- `sprite warp` - Navigation with path options
- `sprite zoom` - Pane focusing with clear feedback

## Compliance Validation

### Automated Checks
The accessibility module includes validation functions:

```rust
use crate::utils::accessibility::validate_wcag_compliance;

let issues = validate_wcag_compliance(output_text);
if !issues.is_empty() {
    // Handle accessibility issues
}
```

### Manual Review
- Regular accessibility audits
- User testing with assistive technologies
- Community feedback incorporation
- Continuous improvement process

## Contributing

When contributing to Sprite, please ensure:
- All user-facing text is accessibility-compliant
- Error messages include actionable suggestions
- Color is never the only information carrier
- Screen reader compatibility is maintained
- Keyboard-only operation is supported

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [NO_COLOR Specification](https://no-color.org/)
- [Command Line Interface Guidelines](https://clig.dev/)
- [Accessibility in Open Source](https://a11yproject.com/)

## Support

For accessibility-related issues or suggestions:
- File an issue with the `accessibility` label
- Include environment details and assistive technology used
- Provide specific reproduction steps
- Share suggestions for improvement

---

*Last Updated: 2025-10-20*
*Version: 1.0*
*Compliance: WCAG 2.1 AA*