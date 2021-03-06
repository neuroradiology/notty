use std::ops::{Deref, DerefMut};

use datatypes::{Region, SaveGrid, SplitKind, ResizeRule};
use terminal::char_grid::CharGrid;

mod panel;
mod section;
mod iter;
mod ring;

pub use self::iter::{Cells, Panels};

use self::section::ScreenSection;

const E_ACTIVE: &'static str = "Active screen section must exist.";

pub trait GridFill {
    fn new(u32, u32, bool) -> Self;
    fn resize(&mut self, Region);
}

impl GridFill for CharGrid {
    fn new(width: u32, height: u32, expand: bool) -> CharGrid {
        CharGrid::new(width, height, expand)
    }
    fn resize(&mut self, area: Region) { self.resize_window(area); }
}

impl GridFill for Region {
    fn new(width: u32, height: u32, _: bool) -> Region {
        Region::new(0, 0, width, height)
    }
    fn resize(&mut self, area: Region) { *self = Region::new(0, 0, area.width(), area.height()) }
}

pub struct Screen {
    active: u64,
    screen: ScreenSection,
}

impl Screen {

    pub fn new(width: u32, height: u32) -> Screen {
        Screen {
            active: 0,
            screen: ScreenSection::new(0, Region::new(0, 0, width, height), true),
        }
    }

    pub fn area(&self) -> Region {
        self.screen.area()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen.resize(Region::new(0, 0, width, height), ResizeRule::Percentage);
    }

    pub fn switch(&mut self, tag: u64) {
        if self.find(Some(tag)).map_or(false, ScreenSection::is_grid) {
            self.active = tag;
        }
    }

    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 split_tag: Option<u64>, l_tag: u64, r_tag: u64, retain_offscreen_state: bool) {
        self.find_mut(split_tag).map(|section| section.split(save, kind, rule, l_tag, r_tag,
                                                             retain_offscreen_state));
        if split_tag.map_or(true, |tag| tag == self.active) {
            self.active = match save {
                SaveGrid::Left  => l_tag,
                SaveGrid::Right => r_tag,
            };
        }
    }

    pub fn unsplit(&mut self, save: SaveGrid, tag: u64) {
        if let Some((left, right)) = self.screen.find(tag).and_then(ScreenSection::children) {
            if self.active == left.tag() || self.active == right.tag() {
                self.active = tag;
            }
        }
        self.find_mut(Some(tag)).map(|section| section.unsplit(save));
    }

    pub fn adjust_split(&mut self, tag: u64, kind: SplitKind, rule: ResizeRule) {
        self.find_mut(Some(tag)).map(|section| section.adjust_split(kind, rule));
    }

    pub fn push(&mut self, tag: Option<u64>, retain_offscreen_state: bool) {
        self.find_mut(tag).map(|section| section.push(retain_offscreen_state));
    }

    pub fn pop(&mut self, tag: Option<u64>) {
        self.find_mut(tag).map(ScreenSection::pop);
    }

    pub fn rotate_down(&mut self, tag: Option<u64>) {
        self.find_mut(tag).map(ScreenSection::rotate_down);
    }

    pub fn rotate_up(&mut self, tag: Option<u64>) {
        self.find_mut(tag).map(ScreenSection::rotate_up);
    }

    pub fn cells(&self) -> Cells {
        self.screen.cells()
    }

    pub fn panels(&self) -> Panels {
        self.screen.panels()
    }

    fn find(&self, tag: Option<u64>) -> Option<&ScreenSection> {
        self.screen.find(tag.unwrap_or(self.active))
    }

    fn find_mut(&mut self, tag: Option<u64>) -> Option<&mut ScreenSection> {
        self.screen.find_mut(tag.unwrap_or(self.active))
    }

}

impl Deref for Screen {
    type Target = CharGrid;
    fn deref(&self) -> &CharGrid {
        self.find(None).expect(E_ACTIVE).grid()
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut CharGrid {
        self.find_mut(None).expect(E_ACTIVE).grid_mut()
    }
}
