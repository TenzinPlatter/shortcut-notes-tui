use gh_tui::{
    app::App,
    block::{CounterBlock, ParagraphBlock},
    view::ViewBuilder,
};
use ratatui::layout::{Constraint, Direction};

fn main() -> anyhow::Result<()> {
    let counters = ViewBuilder::default()
        .add_section(CounterBlock::default())
        .add_section(CounterBlock::default())
        .direction(ratatui::layout::Direction::Horizontal)
        .build();

    let instructions = ViewBuilder::default()
        .add_section(ParagraphBlock::default())
        .build();

    let view = ViewBuilder::default()
        .add_section_with_constraint(counters, Constraint::Percentage(90))
        .add_section_with_constraint(instructions, Constraint::Percentage(10))
        .direction(Direction::Vertical)
        .build();

    // runs closure, providing a terminal instance once closed, terminal is cleaned up
    // then we can return any errors and they will be seen without leftover tui
    ratatui::run(|terminal| App::from(view).run(terminal))?;
    Ok(())
}
