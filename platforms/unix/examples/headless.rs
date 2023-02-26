use accesskit::{
    Action, ActionHandler, ActionRequest, DefaultActionVerb, Live, Node, NodeBuilder, NodeClassSet, NodeId, Rect,
    Role, Tree, TreeUpdate,
};
use accesskit_unix::Adapter;
use std::{
    num::NonZeroU128,
    sync::{Arc, Mutex},
};

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(3) });
const ANNOUNCEMENT_ID: NodeId = NodeId(unsafe { NonZeroU128::new_unchecked(4) });
const INITIAL_FOCUS: NodeId = BUTTON_1_ID;

const BUTTON_1_RECT: Rect = Rect {
    x0: 20.0,
    y0: 20.0,
    x1: 100.0,
    y1: 60.0,
};

const BUTTON_2_RECT: Rect = Rect {
    x0: 20.0,
    y0: 60.0,
    x1: 100.0,
    y1: 100.0,
};

fn build_button(id: NodeId, name: &str, classes: &mut NodeClassSet) -> Node {
    let rect = match id {
        BUTTON_1_ID => BUTTON_1_RECT,
        BUTTON_2_ID => BUTTON_2_RECT,
        _ => unreachable!(),
    };

    let mut builder = NodeBuilder::new(Role::Button);
    builder.set_bounds(rect);
    builder.set_name(name);
    builder.add_action(Action::Focus);
    builder.set_default_action_verb(DefaultActionVerb::Click);
    builder.build(classes)
}

fn build_announcement(text: &str, classes: &mut NodeClassSet) -> Node {
    let mut builder = NodeBuilder::new(Role::StaticText);
    builder.set_name(text);
    builder.set_live(Live::Polite);
    builder.build(classes)
}

struct State {
    focus: NodeId,
    is_window_focused: bool,
    announcement: Option<String>,
    node_classes: NodeClassSet,
}

impl State {
    fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            focus: INITIAL_FOCUS,
            is_window_focused: false,
            announcement: None,
            node_classes: NodeClassSet::new(),
        }))
    }

    fn focus(&self) -> Option<NodeId> {
        self.is_window_focused.then_some(self.focus)
    }

    fn build_root(&mut self) -> Node {
        let mut builder = NodeBuilder::new(Role::Window);
        builder.set_children(vec![BUTTON_1_ID, BUTTON_2_ID]);
        if self.announcement.is_some() {
            builder.push_child(ANNOUNCEMENT_ID);
        }
        builder.set_name(WINDOW_TITLE);
        builder.build(&mut self.node_classes)
    }

    fn build_initial_tree(&mut self) -> TreeUpdate {
        let root = self.build_root();
        let button_1 = build_button(BUTTON_1_ID, "Button 1", &mut self.node_classes);
        let button_2 = build_button(BUTTON_2_ID, "Button 2", &mut self.node_classes);
        let mut result = TreeUpdate {
            nodes: vec![
                (WINDOW_ID, root),
                (BUTTON_1_ID, button_1),
                (BUTTON_2_ID, button_2),
            ],
            tree: Some(Tree::new(WINDOW_ID)),
            focus: self.focus(),
        };
        if let Some(announcement) = &self.announcement {
            result.nodes.push((
                ANNOUNCEMENT_ID,
                build_announcement(announcement, &mut self.node_classes),
            ));
        }
        result
    }

    fn update_focus(&mut self, adapter: &Adapter) {
        adapter.update(TreeUpdate {
            nodes: vec![],
            tree: None,
            focus: self.focus(),
        });
    }
}

struct NullActionHandler;

impl ActionHandler for NullActionHandler {
    fn do_action(&self, _request: ActionRequest) {
    }
}

fn main() {
    let state = State::new();

    let adapter = {
        let state = Arc::clone(&state);
        Adapter::new(
            "".into(),
            "".into(),
            "".into(),
            move || {
                let mut state = state.lock().unwrap();
                state.build_initial_tree()
            },
            Box::new(NullActionHandler {}),
        )
    }.unwrap();

    {
        let mut state = state.lock().unwrap();
        state.is_window_focused = true;
        state.update_focus(&adapter);
    }

    std::thread::park();
    drop(adapter);
}
