use gh_tui::{
    app::App,
    block::{CounterBlock, ParagraphBlock},
    view::ViewBuilder,
};
use ratatui::layout::{Constraint, Direction};

fn main() -> anyhow::Result<()> {
    let counters = ViewBuilder::default()
        .add_selectable(CounterBlock::default())
        .add_selectable(CounterBlock::default())
        .direction(ratatui::layout::Direction::Horizontal)
        .build();

    let instructions = ViewBuilder::default()
        .add_non_selectable(ParagraphBlock::instructions())
        .build();

    let view = ViewBuilder::default()
        .add_selectable_with_constraint(counters, Constraint::Percentage(80))
        .add_non_selectable_with_constraint(instructions, Constraint::Percentage(20))
        .direction(Direction::Vertical)
        .build();

    // runs closure, providing a terminal instance once closed, terminal is cleaned up
    // then we can return any errors and they will be seen without leftover tui
    ratatui::run(|terminal| App::from(view).run(terminal))?;
    Ok(())
}
