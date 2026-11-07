//! Lays out Hall of fame screenshots in a grid.

use gtk::prelude::*;

use super::ScreenshotCard;
use crate::catalog::{Catalog, Screenshot};

/// The Hall of fame grid is four columns wide.
const WALL_COLUMNS: i32 = 4;

/// Fills `grid` with `screenshots`. The first one is large; the rest are small.
/// Subjects with a real screenshot in the catalog show it in place of the drawing.
pub fn fill_wall_grid(grid: &gtk::Grid, screenshots: &[&Screenshot], catalog: &Catalog) {
    let cards: Vec<ScreenshotCard> = screenshots
        .iter()
        .map(|screenshot| wall_card(screenshot, catalog))
        .collect();
    fill_with_cards(grid, &cards);
}

/// Lays out ready-made cards: the first large, the rest small.
pub fn fill_with_cards(grid: &gtk::Grid, cards: &[ScreenshotCard]) {
    for (index, card) in cards.iter().enumerate() {
        let cell = wall_cell(index);
        grid.attach(card, cell.column, cell.row, cell.width, cell.height);
    }

    if cards.len() == 1 {
        keep_lone_screenshot_at_half_width(grid);
    }
}

fn wall_card(screenshot: &Screenshot, catalog: &Catalog) -> ScreenshotCard {
    match catalog.subject_screenshot(&screenshot.subject) {
        Some((name, hosted)) => ScreenshotCard::with_hosted(Some(screenshot), name, hosted, &screenshot.link),
        None => ScreenshotCard::new(screenshot),
    }
}

/// The grid's columns share its width, so a lone screenshot would fill all of it
/// and grow very tall. An empty cell beside it keeps it to half the width.
fn keep_lone_screenshot_at_half_width(grid: &gtk::Grid) {
    let empty_cell = gtk::Box::new(gtk::Orientation::Vertical, 0);
    grid.attach(&empty_cell, 2, 0, 2, 2);
}

/// Where a screenshot sits in the grid.
#[derive(Debug, PartialEq, Eq)]
struct WallCell {
    column: i32,
    row: i32,
    width: i32,
    height: i32,
}

/// The first screenshot is two columns wide and two rows tall.
/// The next four fill the cells beside it, and the rest continue in rows below.
fn wall_cell(index: usize) -> WallCell {
    let index = index as i32;

    if index == 0 {
        return WallCell { column: 0, row: 0, width: 2, height: 2 };
    }

    let small_index = index - 1;
    if small_index < 4 {
        return WallCell { column: 2 + small_index % 2, row: small_index / 2, width: 1, height: 1 };
    }

    let below_index = small_index - 4;
    WallCell {
        column: below_index % WALL_COLUMNS,
        row: 2 + below_index / WALL_COLUMNS,
        width: 1,
        height: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_screenshot_is_large() {
        assert_eq!(wall_cell(0), WallCell { column: 0, row: 0, width: 2, height: 2 });
    }

    #[test]
    fn next_four_sit_beside_the_large_one() {
        assert_eq!(wall_cell(1), WallCell { column: 2, row: 0, width: 1, height: 1 });
        assert_eq!(wall_cell(4), WallCell { column: 3, row: 1, width: 1, height: 1 });
    }

    #[test]
    fn the_rest_continue_below() {
        assert_eq!(wall_cell(5), WallCell { column: 0, row: 2, width: 1, height: 1 });
        assert_eq!(wall_cell(9), WallCell { column: 0, row: 3, width: 1, height: 1 });
    }
}
