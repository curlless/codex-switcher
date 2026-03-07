# Design Guidelines

## Desktop GUI Direction

The desktop GUI should feel adjacent to Cursor, not like a clone.

Desired qualities:

- quiet, focused, low-noise interface
- dense information layout with strong hierarchy
- dark-first visual system with restrained contrast
- subtle accent color usage for actionable state, not decoration
- minimal chrome around the core profile-management workflow

## Layout Principles

- Left rail for profile navigation and filters.
- Main pane for profile detail, usage status, and primary actions.
- Utility bar for global status, reload targets, and last refresh time.
- Use cards and grouped surfaces sparingly; prefer clean sections over dashboard clutter.

## Typography

- Prefer a modern grotesk or neo-grotesk UI font with compact metrics.
- Use large contrast jumps between page title, section title, and dense data rows.
- Avoid oversized headings that waste vertical space.

## Color System

- Base surfaces should stay in graphite, charcoal, and muted blue-gray ranges.
- Accent should be a cool electric blue or blue-cyan, used only for selected items, focus, and positive action affordances.
- Warning and failure states should be explicit but not neon-heavy.

## Interaction Style

- One primary action per panel.
- Inline confirmation for low-risk actions.
- Explicit modal confirmation only for destructive operations like profile deletion.
- Fast keyboard navigation is a requirement, not polish.

## Motion

- Use short opacity and translate transitions for list/detail changes.
- Avoid playful or bouncy motion.
- Motion should confirm state transitions, not draw attention to itself.

## Components for MVP

- Profile sidebar item
- Usage headroom badge
- Active profile marker
- Reserve toggle
- Switch action bar
- Reload target segmented control
- Status toast / inline error banner

## Anti-Patterns

- Do not mimic terminal aesthetics inside the GUI.
- Do not overload the home screen with metrics that do not affect switching decisions.
- Do not introduce a second visual language for Codex vs Cursor targets.
