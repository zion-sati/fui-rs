use crate::bindings::ui;
use crate::ffi::{HandleValue, FlexDirection, NodeType, Unit};

pub struct BuiltNode {
    handle: u64,
    children: Vec<BuiltNode>,
    destroyed: bool,
}

impl BuiltNode {
    pub fn new(handle: u64) -> Self {
        Self {
            handle,
            children: Vec::new(),
            destroyed: false,
        }
    }

    pub fn handle(&self) -> u64 {
        self.handle
    }

    pub fn add_child(&mut self, child: BuiltNode) {
        ui::add_child(self.handle, child.handle);
        self.children.push(child);
    }

    pub fn destroy(&mut self) {
        if self.destroyed {
            return;
        }

        for child in self.children.iter_mut().rev() {
            child.destroy();
        }
        self.children.clear();

        if self.handle != HandleValue::Invalid as u64 {
            ui::delete_node(self.handle);
        }
        self.destroyed = true;
    }
}

pub trait Node {
    fn build(&self) -> BuiltNode;
}

#[derive(Default)]
pub struct FlexBox {
    width: Option<(f32, Unit)>,
    height: Option<(f32, Unit)>,
    bg_color: Option<u32>,
    padding: Option<(f32, f32, f32, f32)>,
    flex_direction: Option<FlexDirection>,
    children: Vec<Box<dyn Node>>,
}

impl FlexBox {
    pub fn width(mut self, width: f32, unit: Unit) -> Self {
        self.width = Some((width, unit));
        self
    }

    pub fn height(mut self, height: f32, unit: Unit) -> Self {
        self.height = Some((height, unit));
        self
    }

    pub fn bg_color(mut self, color: u32) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn padding(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.padding = Some((top, right, bottom, left));
        self
    }

    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.flex_direction = Some(direction);
        self
    }

    pub fn child(mut self, node: impl Node + 'static) -> Self {
        self.children.push(Box::new(node));
        self
    }
}

impl Node for FlexBox {
    fn build(&self) -> BuiltNode {
        let mut node = BuiltNode::new(ui::create_node(NodeType::FlexBox as u32));
        if let Some((width, unit)) = self.width {
            ui::set_width(node.handle(), width, unit as u32);
        }
        if let Some((height, unit)) = self.height {
            ui::set_height(node.handle(), height, unit as u32);
        }
        if let Some(color) = self.bg_color {
            ui::set_bg_color(node.handle(), color);
        }
        if let Some((top, right, bottom, left)) = self.padding {
            ui::set_padding(node.handle(), top, right, bottom, left);
        }
        if let Some(direction) = self.flex_direction {
            ui::set_flex_direction(node.handle(), direction as u32);
        }

        for child in &self.children {
            node.add_child(child.build());
        }

        node
    }
}

pub struct TextNode {
    content: String,
    font: Option<(u32, f32)>,
    text_color: Option<u32>,
}

impl TextNode {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            font: None,
            text_color: None,
        }
    }

    pub fn font(mut self, font_id: u32, size: f32) -> Self {
        self.font = Some((font_id, size));
        self
    }

    pub fn text_color(mut self, color: u32) -> Self {
        self.text_color = Some(color);
        self
    }
}

impl Node for TextNode {
    fn build(&self) -> BuiltNode {
        let node = BuiltNode::new(ui::create_node(NodeType::Text as u32));
        ui::set_text(node.handle(), &self.content);
        if let Some((font_id, size)) = self.font {
            ui::set_font(node.handle(), font_id, size);
        }
        if let Some(color) = self.text_color {
            ui::set_text_color(node.handle(), color);
        }
        node
    }
}

pub fn flex_box() -> FlexBox {
    FlexBox::default()
}

pub fn text(content: &str) -> TextNode {
    TextNode::new(content)
}

pub fn row() -> FlexBox {
    FlexBox::default().flex_direction(FlexDirection::Row)
}

pub fn column() -> FlexBox {
    FlexBox::default().flex_direction(FlexDirection::Column)
}

#[allow(dead_code)]
pub fn pixel_unit() -> Unit {
    Unit::Pixel
}
