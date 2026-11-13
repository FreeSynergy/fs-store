//! The click point of the typing test (`.github/workflows/typing-test.yml`, `click-in-window`) lies in the text field "Git address of
//! a source" of `--demo --look standard --view sources --size 900x640`; a click there gives the field the keyboard and a typed
//! character reaches the model. The layout moves whenever the page above the field changes (the strip of messages, the
//! sections), and then the typing test shows "no typed glyph": this test turns that into a red test here.
//!
//! The window is laid out and driven without opening one (Blinc's tree and router, as the window's event loop does).

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use fs_components::blinc::core::owner::BuildScope;
use fs_components::blinc::core::reactive::global_graph;
use fs_components::blinc::core::{BlincContextState, HookState};
use fs_components::blinc::layout::css_parser::Stylesheet;
use fs_components::blinc::layout::event_router::{EventRouter, MouseButton};
use fs_components::blinc::layout::prelude::RenderTree;
use fs_components::blinc::layout::tree::LayoutNodeId;
use fs_components::blinc::layout::widgets::text_input::{
    blur_all_text_inputs, focused_text_input_node_id,
};
use fs_components::blinc::theme::{ColorScheme, ThemeState};
use fs_store_app::demo::parts;
use fs_store_app::model::{StoreModel, View};
use fs_store_app::view::store_app;

const SIZE: (f32, f32) = (900.0, 640.0);
const WORKFLOW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/typing-test.yml"
);

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Left, top, right, bottom of a rectangle.
type Edges = (f32, f32, f32, f32);

struct Window {
    tree: RenderTree,
    router: EventRouter,
    model: Arc<Mutex<StoreModel>>,
    _serial: MutexGuard<'static, ()>,
}

/// The window as `fs-store --demo --look standard --view sources --size 900x640` builds it.
fn window() -> Window {
    let serial = ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner);
    // the field of a test before may still have the keyboard (process-wide in Blinc)
    blur_all_text_inputs();
    let parts = parts(true);
    let mut model = StoreModel::new(parts.backend).with_updates(parts.updates);
    fs_components_render::use_style(None);
    model.refresh();
    model.show(View::Sources);
    let model = Arc::new(Mutex::new(model));
    let app = store_app(&model, &parts.launcher).unwrap();
    let bundle = fs_components_render::bundle(&app.styles());
    ThemeState::init(bundle.clone(), ColorScheme::Light);
    if !BlincContextState::is_initialized() {
        BlincContextState::init_with_callback(
            global_graph(),
            Arc::new(Mutex::new(HookState::new())),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
            Arc::new(|ids: &[_]| {
                fs_components::blinc::layout::check_stateful_deps(ids);
            }),
        );
    }
    fs_components::blinc::layout::init_text_measurer();
    let mut scope = BuildScope::new();
    let pass = scope.begin();
    fs_components::blinc::layout::key::reset_call_counters();
    let mut tree = RenderTree::from_element(&app.build(SIZE.0, SIZE.1));
    let vars = ThemeState::get().to_css_variable_map();
    let css = bundle.css_sources.join("\n");
    tree.set_stylesheet(Stylesheet::parse_with_variables(&css, &vars).unwrap());
    tree.apply_stylesheet_layout_overrides();
    tree.apply_stylesheet_base_styles();
    tree.compute_layout(SIZE.0, SIZE.1);
    scope.finish(pass);
    Window {
        tree,
        router: EventRouter::new(),
        model,
        _serial: serial,
    }
}

impl Window {
    /// Press and release the left button at a point, with nothing in between.
    fn click(&mut self, (x, y): (f32, f32)) {
        let moved = self.router.on_mouse_move(&self.tree, x, y);
        self.send(moved, x, y);
        let down = self
            .router
            .on_mouse_down(&self.tree, x, y, MouseButton::Left);
        self.send(down, x, y);
        let up = self.router.on_mouse_up(&self.tree, x, y, MouseButton::Left);
        self.send(up, x, y);
    }

    fn send(&mut self, events: Vec<(LayoutNodeId, u32)>, x: f32, y: f32) {
        for (node, event) in events {
            self.tree.dispatch_event(node, event, x, y);
        }
    }

    /// The rectangle of the one text field of the page: found by clicking on a grid until a field takes the keyboard.
    fn field(&mut self) -> Edges {
        let mut y = 4.0;
        while y < SIZE.1 {
            let mut x = 4.0;
            while x < SIZE.0 {
                blur_all_text_inputs();
                self.click((x, y));
                if let Some(node) = focused_text_input_node_id() {
                    let b = self.tree.get_absolute_bounds(node).expect("bounds");
                    blur_all_text_inputs();
                    return (b.x, b.y, b.x + b.width, b.y + b.height);
                }
                x += 8.0;
            }
            y += 8.0;
        }
        panic!("no text field on the sources page");
    }
}

/// The point of `click-in-window` in the workflow of the typing test.
fn workflow_point() -> (f32, f32) {
    let yaml = std::fs::read_to_string(WORKFLOW).expect("typing-test.yml");
    let line = yaml
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("click-in-window:"))
        .expect("a click-in-window line");
    let value = line.trim_start_matches("click-in-window:").trim();
    let (x, y) = value
        .trim_matches('"')
        .split_once(',')
        .expect("a point x,y");
    (x.trim().parse().unwrap(), y.trim().parse().unwrap())
}

/// The click is measured from the window's top left corner, title bar included (`KWin`'s, as in the typing test); the bar is 0 px
/// high without decoration and about 30 to 36 px with the Breeze one, so the point must hold for the whole range up to this.
const TITLE_BAR_MAX: f32 = 40.0;
/// The typing test judges the inside of the field: border, ring and rounded corners (6 px across, 4 px down) are left out.
const INSET: (f32, f32) = (6.0, 4.0);

/// The click point in the page, for a title bar of `bar` px.
fn in_page((x, y): (f32, f32), bar: f32) -> (f32, f32) {
    (x, y - bar)
}

/// Is `point` in the inside of the field (inset from the edges)?
fn inside((left, top, right, bottom): Edges, (x, y): (f32, f32)) -> bool {
    (left + INSET.0..right - INSET.0).contains(&x) && (top + INSET.1..bottom - INSET.1).contains(&y)
}

fn holds_for_every_title_bar(field: Edges, point: (f32, f32)) -> bool {
    [0.0, TITLE_BAR_MAX / 2.0, TITLE_BAR_MAX]
        .into_iter()
        .all(|bar| inside(field, in_page(point, bar)))
}

#[test]
fn the_click_point_of_the_typing_test_lies_in_the_field() {
    let mut w = window();
    let field = w.field();
    let point = workflow_point();
    assert!(
        holds_for_every_title_bar(field, point),
        "click-in-window {point:?} misses the field {field:?} (with a title bar of 0 to {TITLE_BAR_MAX} px); a good point is {:?}",
        (
            f32::midpoint(field.0, field.2),
            f32::midpoint(field.1, field.3) + TITLE_BAR_MAX / 2.0
        )
    );
}

#[test]
fn a_click_there_gives_the_field_the_keyboard_and_a_character_reaches_the_model() {
    for bar in [0.0, TITLE_BAR_MAX] {
        let mut w = window();
        assert!(focused_text_input_node_id().is_none());
        w.click(in_page(workflow_point(), bar));
        let node = focused_text_input_node_id().expect("the click did not focus a field");
        w.tree
            .dispatch_text_input_event(node, 'g', false, false, false, false);
        let typed = w.model.lock().unwrap().new_source().to_owned();
        assert_eq!(typed, "g", "title bar {bar} px");
        blur_all_text_inputs();
    }
}

#[test]
fn a_point_that_misses_the_field_is_refused() {
    let mut w = window();
    let field = w.field();
    // the check itself: the corner of the window and a point past the field are no points of it, and a click there
    // leaves the keyboard alone
    assert!(!holds_for_every_title_bar(field, (2.0, 2.0)));
    assert!(!holds_for_every_title_bar(
        field,
        (field.2 + 1.0, field.3 + 1.0)
    ));
    // the point of 0.0.16 before the fix (300,190) sat on the field's top border once a title bar was added
    assert!(!holds_for_every_title_bar(field, (300.0, 190.0)));
    w.click((2.0, 2.0));
    assert!(focused_text_input_node_id().is_none());
}
