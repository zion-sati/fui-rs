use fui_rs::app::Application;
use fui_rs::component::Component;
use fui_rs::ffi;
use fui_rs::state::{state, State};
use fui_rs::node::{flex_box, Node};

struct RedBoxComponent {
    color: State<u32>,
}

impl RedBoxComponent {
    fn new() -> Self {
        Self {
            color: state(0xff0000ff),
        }
    }
}

impl Component for RedBoxComponent {
    fn render(&self) -> Box<dyn Node> {
        Box::new(
            flex_box()
                .width(320.0, fui_rs::UiUnit::Pixel)
                .height(220.0, fui_rs::UiUnit::Pixel)
                .bg_color(self.color.get()),
        )
    }
}

#[test]
fn application_builds_and_commits_a_red_box() {
    ffi::test::reset();

    Application::run(RedBoxComponent::new);

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, ffi::UiCall::Reset)));
    assert!(calls.iter().any(|call| matches!(call, ffi::UiCall::ResizeWindow { .. })));
    assert!(calls.iter().any(|call| matches!(call, ffi::UiCall::CreateNode { node_type, .. } if *node_type == fui_rs::UiNodeType::FlexBox as u32)));
    assert!(calls.iter().any(|call| matches!(call, ffi::UiCall::SetBackgroundColor { color, .. } if *color == 0xff0000ff)));
    assert!(calls.iter().any(|call| matches!(call, ffi::UiCall::CommitFrame)));
}
