# TODO

- [ ] Group stories by iteration
- [ ] Add (fuzzy?) search functionality to action_menu, and numbers? Could be like code actions
- [ ] Fix active story not saving properly
- [ ] Ask task queue with notifications for async tasks like api fetching etc. should be able to display multiple tasks at once
- [ ] Some task is still blocking on close sometimes, maybe story fetching idk, need to investigate

- [x] Make api fetching quit automatically if closing tui before they are finished
- [x] Make fzf for git worktree cancellable without crashing
- [x] notes system
- [x] move cache out of ~/.config
- [x] add borders between list items (this requires rewriting the List widget)
- [x] Migrate story list to tui-widget-list
- [x] Move action menu to tui-widget-list
- [x] implement tmux session management tied to ticket
- [x] add git worktree integration (will use a fzf process over all dirs in Repositories that have a .git dir (or maybe package.xml if i want to search by package?) and then create a worktree for it)
- [x] Refactor msgs/cmds to not take entire Story/Iteration objects, just what is needed to avoid cloning everything
- [x] Handle multiple active iterations
- [x] Move active_story out of story list state into data state
- [x] Keybind to open ticket in browser

- [ ] add shortcut integration:
  - [ ] Edit ticket comments
  - [ ] attach note as file to ticket
  - [ ] Change ticket state
  - [ ] Update days taken (do automatically when putting ticket to finished?)
  - [ ] Blocking/blockers indicators (just amount) 
  - [ ] TODO: add todo points for epic/iteration integration
  - [x] Edit ticket description

- [ ] windows:
  - [ ] Search
  - [ ] Filters - based on what is showing
  - [ ] Add iteration page
