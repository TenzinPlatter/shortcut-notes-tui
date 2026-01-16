# Work Notes Management System Design

**Date:** 2026-01-16
**Purpose:** Fast-capture note system integrated with Shortcut project management

## Problem Statement

Current notes setup has multiple pain points:
- **Capture friction**: Deciding where to put notes and switching contexts slows down note-taking
- **Context loss**: Notes lack connection to work context (tickets, iterations, epics)
- **Disorganization**: Hard to find notes later when needed

Primary use cases are note capture during:
- Meetings (need quick jotting while listening)
- Coding sessions (bugs, TODOs, architecture thoughts)
- Throughout the day (random thoughts, reminders)

Primary environments: terminal/nvim and browser.

## Solution Overview

Build an integrated TUI workspace manager that tracks Shortcut tickets, manages tmux sessions, and provides fast note capture with automatic context tagging. Notes remain Obsidian-compatible markdown files with rich frontmatter for filtering.

**Core principle**: Optimize for capture speed by eliminating decisions and context switching.

## System Architecture

### Component Diagram

```
┌─────────────────┐      ┌──────────────┐      ┌─────────────┐
│   CLI Command   │─────▶│  State File  │◀─────│  TUI App    │
│   `note ...`    │      │  (JSON/TOML) │      │             │
└─────────────────┘      └──────────────┘      └─────────────┘
         │                      │                      │
         │                      ▼                      │
         │               ┌──────────────┐              │
         └──────────────▶│ Notes Index  │◀─────────────┘
                         │  (SQLite)    │
                         └──────────────┘
                                │
                                ▼
                         ┌──────────────┐
                         │  Markdown    │
                         │   Files      │
                         └──────────────┘
                                ▲
                                │
                         ┌──────────────┐
                         │  Obsidian    │
                         │    nvim      │
                         └──────────────┘

         ┌──────────────┐
         │  Shortcut    │
         │     API      │
         └──────────────┘
                ▲
                │
         ┌──────────────┐
         │  TUI App     │
         │  (API sync)  │
         └──────────────┘
```

### Components

**1. TUI Application**
Central workspace manager with three main views:
- **Tickets view**: Browse Shortcut stories, set active ticket, create notes for tickets, manage tmux sessions
- **Notes browser**: Filter/search notes by ticket, epic, iteration, date, type, tags. Opens notes in $EDITOR
- **Sessions view**: List tmux sessions with ticket associations

**2. CLI Command**
Fast note capture from anywhere:
```bash
note "quick thought"
note --ticket sc-12345 "specific ticket note"
note --type meeting
```
Reads current context from state file, creates note with auto-populated frontmatter.

**3. State File** (`~/.config/worknotes/state.json`)
Shared state between TUI and CLI:
- Active ticket ID
- Active epic ID
- Current iteration number
- Shortcut workspace/API token
- Ticket → tmux session mapping
- Last sync timestamp

**4. Notes Index** (SQLite)
Fast searchable index of all notes:
- Schema: `notes(id, path, created, ticket, epic, iteration, type, tags, title, content_preview)`
- Indexed on: ticket, epic, iteration, type, created
- Full-text search on title + content_preview
- Rebuilt on TUI startup, updated on note creation

**5. Markdown Files**
Obsidian-compatible notes with rich frontmatter:
- Storage: `notes/YYYY/MM/DD-HHmmss-slug.md`
- Format: Standard markdown with YAML frontmatter
- Compatible with existing Obsidian nvim setup

## Data Model

### Note Frontmatter

```yaml
---
id: 2026-01-16-153042-auth-bug-fix
created: 2026-01-16T15:30:42Z
ticket: sc-12345
epic: sc-500
iteration: 25
type: code-note  # meeting, idea, todo
tags: [auth, bug, backend]
aliases: []
---
```

**Field Descriptions:**
- `id`: Unique identifier (timestamp + slug)
- `created`: ISO timestamp for chronological sorting
- `ticket`: Shortcut story ID (optional, auto-populated from active ticket)
- `epic`: Shortcut epic ID (optional, pulled from ticket via API)
- `iteration`: Iteration number (from Shortcut or manual)
- `type`: Note category for filtering (code-note, meeting, idea, todo)
- `tags`: Free-form tags for additional organization
- `aliases`: Alternative names for Obsidian wikilink resolution

### File Storage

Simple chronological directory structure:
```
notes/
  2026/
    01/
      16-153042-auth-bug-fix.md
      16-142230-backend-meeting.md
    02/
      ...
  YYYY/
    MM/
      ...
```

Benefits:
- Easy to browse by date if needed
- Obsidian handles this structure fine
- TUI index makes physical structure irrelevant for search
- No complex directory decisions during capture

## Workflows

### Quick Capture from Terminal (Coding Flow)

```bash
# User is debugging, finds issue
$ note "auth token expiry set to 0 in dev config"
# → Creates note with current ticket/epic/iteration from state file
# → Returns immediately to terminal
```

Process:
1. CLI reads `~/.config/worknotes/state.json` for active ticket context
2. Generates frontmatter with ticket, epic, iteration
3. Creates file: `notes/2026/01/16-153042-auth-token-expiry.md`
4. Updates SQLite index
5. Returns control to user

### Meeting Notes (TUI Flow)

1. Switch to TUI (tmux hotkey or command)
2. Active ticket is highlighted in tickets pane
3. Press `n` for new note
4. TUI prompts for type: (m)eeting, (c)ode, (i)dea, (t)odo
5. Press `m` for meeting
6. TUI creates note with frontmatter, opens in $EDITOR
7. User writes meeting notes using Obsidian nvim features (wikilinks, etc.)
8. Save and exit, return to TUI

### Finding Old Notes

1. In TUI, switch to notes browser view
2. Press `/` to enter filter mode
3. Type filter: `ticket:sc-12345 type:meeting`
4. Browse filtered list
5. Press `Enter` on note to open in $EDITOR
6. Wikilinks work as normal in Obsidian nvim

## Shortcut Integration

**One-way sync**: Shortcut → TUI/Notes (read-only)

**What gets synced:**
- Stories (tickets) for current iteration
- Epic associations for stories
- Iteration numbers
- Story metadata (title, status, owner, etc.)

**What doesn't sync:**
- Notes don't write back to Shortcut
- No automatic comment creation
- Manual process if you want to link notes in Shortcut

**API Usage:**
- TUI fetches stories on startup or manual refresh
- When setting active ticket, TUI queries API for epic + iteration
- Cached locally in state file to minimize API calls
- Auto-populate metadata when creating notes

## Benefits

**Eliminates capture friction:**
- No decision fatigue (where to put this note?)
- No context switching (CLI from anywhere, TUI from workspace)
- Auto-populated metadata (ticket, epic, iteration from current context)

**Rich organization without effort:**
- All notes tagged with work context automatically
- Filter by any dimension: ticket, epic, iteration, time, type
- Full-text search across all notes
- Multiple views of the same data

**Obsidian compatibility:**
- Existing nvim setup continues to work
- Wikilinks function normally
- Can use Obsidian graph view, backlinks, etc.
- Just adds metadata, doesn't break anything

**Integrated workflow:**
- Tickets, sessions, and notes in one interface
- Tmux sessions tied to tickets
- Context always available, never hunting for info

## Implementation Considerations

**Technology choices** (TBD based on existing TUI):
- TUI framework: (what are you using currently?)
- CLI: Shell script or compiled binary
- State format: JSON or TOML
- Index: SQLite for robustness

**Future enhancements** (not initial scope):
- Note templates per type
- Review workflows (inbox processing)
- Daily/weekly note aggregation
- Export to different formats
- Mobile capture integration
- Bi-directional Shortcut sync

**Out of scope:**
- Automatic alias generation
- Rich markdown rendering in TUI (use $EDITOR)
- Wikilink parsing in TUI (Obsidian nvim handles it)
- Note versioning/history
- Collaborative features
