# TODO

- [x] notes system
- [x] move cache out of ~/.config
- [x] add borders between list items (this requires rewriting the List widget)
- [x] implement tmux session management tied to ticket
- [x] add git worktree integration (will use a fzf process over all dirs in Repositories that have a .git dir (or maybe package.xml if i want to search by package?) and then create a worktree for it)
- [x] Refactor msgs/cmds to not take entire Story/Iteration objects, just what is needed to avoid cloning everything
- [ ] Fetch owned stories, then filter by their iteration and see which of those are in iterations that are currently active as we have multiple active iterations now
- [x] Move active_story out of story list state into data state
- [ ] Add (fuzzy?) search functionality to action_menu, and numbers? Could be like code actions
- [ ] Move action menu to use custom list
- add shortcut integration:
  - [x] Edit ticket description
  - [ ] Edit ticket comments
  - [ ] Change ticket state
  - [ ] Update days taken (do automatically when putting ticket to finished?)
  - [ ] TODO: add points for epic/iteration integration
