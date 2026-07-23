export interface EnumMemberSpec {
  readonly name: string;
  readonly source: string;
}

export interface EnumSpec {
  readonly name: string;
  readonly source: "ui" | "core" | "host" | "constant";
  readonly sourceEnum?: string;
  readonly members: readonly EnumMemberSpec[];
}

export const canonicalEnumSpecs: readonly EnumSpec[] = [
  { name: "HandleValue", source: "constant", members: [{ name: "Invalid", source: "UI_INVALID_HANDLE" }] },
  {
    name: "NodeType",
    source: "ui",
    sourceEnum: "UiNodeType",
    members: [
      { name: "FlexBox", source: "UI_NODE_FLEX_BOX" },
      { name: "Text", source: "UI_NODE_TEXT" },
      { name: "Image", source: "UI_NODE_IMAGE" },
      { name: "Svg", source: "UI_NODE_SVG" },
      { name: "ScrollView", source: "UI_NODE_SCROLLVIEW" },
      { name: "Grid", source: "UI_NODE_GRID" },
    ],
  },
  {
    name: "Unit",
    source: "ui",
    sourceEnum: "UiSizeUnit",
    members: [
      { name: "Pixel", source: "UI_SIZE_UNIT_PIXEL" },
      { name: "Auto", source: "UI_SIZE_UNIT_AUTO" },
      { name: "Percent", source: "UI_SIZE_UNIT_PERCENT" },
    ],
  },
  {
    name: "GridUnit",
    source: "ui",
    sourceEnum: "UiGridUnit",
    members: [
      { name: "Pixel", source: "UI_GRID_UNIT_PIXEL" },
      { name: "Auto", source: "UI_GRID_UNIT_AUTO" },
      { name: "Star", source: "UI_GRID_UNIT_STAR" },
    ],
  },
  {
    name: "PositionType",
    source: "ui",
    sourceEnum: "UiPositionType",
    members: [
      { name: "Relative", source: "UI_POSITION_RELATIVE" },
      { name: "Absolute", source: "UI_POSITION_ABSOLUTE" },
    ],
  },
  {
    name: "Visibility",
    source: "ui",
    sourceEnum: "UiVisibility",
    members: [
      { name: "Normal", source: "UI_VISIBILITY_NORMAL" },
      { name: "Hidden", source: "UI_VISIBILITY_HIDDEN" },
      { name: "Collapsed", source: "UI_VISIBILITY_COLLAPSED" },
    ],
  },
  {
    name: "FlexDirection",
    source: "ui",
    sourceEnum: "UiFlexDirection",
    members: [
      { name: "Column", source: "UI_FLEX_DIRECTION_COLUMN" },
      { name: "Row", source: "UI_FLEX_DIRECTION_ROW" },
    ],
  },
  {
    name: "FlexWrap",
    source: "ui",
    sourceEnum: "UiFlexWrap",
    members: [
      { name: "NoWrap", source: "UI_FLEX_WRAP_NO_WRAP" },
      { name: "Wrap", source: "UI_FLEX_WRAP_WRAP" },
      { name: "WrapReverse", source: "UI_FLEX_WRAP_WRAP_REVERSE" },
    ],
  },
  {
    name: "JustifyContent",
    source: "ui",
    sourceEnum: "UiJustifyContent",
    members: [
      { name: "Start", source: "UI_JUSTIFY_START" },
      { name: "Center", source: "UI_JUSTIFY_CENTER" },
      { name: "End", source: "UI_JUSTIFY_END" },
    ],
  },
  {
    name: "AlignItems",
    source: "ui",
    sourceEnum: "UiAlignItems",
    members: [
      { name: "Start", source: "UI_ALIGN_ITEMS_START" },
      { name: "Center", source: "UI_ALIGN_ITEMS_CENTER" },
      { name: "End", source: "UI_ALIGN_ITEMS_END" },
      { name: "Stretch", source: "UI_ALIGN_ITEMS_STRETCH" },
      { name: "None", source: "UI_ALIGN_ITEMS_NONE" },
    ],
  },
  {
    name: "AlignSelf",
    source: "ui",
    sourceEnum: "UiAlignSelf",
    members: [
      { name: "Auto", source: "UI_ALIGN_SELF_AUTO" },
      { name: "Start", source: "UI_ALIGN_SELF_START" },
      { name: "Center", source: "UI_ALIGN_SELF_CENTER" },
      { name: "End", source: "UI_ALIGN_SELF_END" },
      { name: "Stretch", source: "UI_ALIGN_SELF_STRETCH" },
    ],
  },
  {
    name: "BorderStyle",
    source: "core",
    sourceEnum: "EdBorderStyle",
    members: [
      { name: "Solid", source: "ED_BORDER_SOLID" },
      { name: "Dashed", source: "ED_BORDER_DASHED" },
      { name: "Dotted", source: "ED_BORDER_DOTTED" },
    ],
  },
  {
    name: "ObjectFit",
    source: "core",
    sourceEnum: "EdObjectFit",
    members: [
      { name: "Fill", source: "ED_OBJECT_FIT_FILL" },
      { name: "Contain", source: "ED_OBJECT_FIT_CONTAIN" },
      { name: "Cover", source: "ED_OBJECT_FIT_COVER" },
      { name: "None", source: "ED_OBJECT_FIT_NONE" },
      { name: "ScaleDown", source: "ED_OBJECT_FIT_SCALE_DOWN" },
    ],
  },
  {
    name: "ImageSamplingKind",
    source: "core",
    sourceEnum: "EdImageSampling",
    members: [
      { name: "Linear", source: "ED_IMAGE_SAMPLING_LINEAR" },
      { name: "Nearest", source: "ED_IMAGE_SAMPLING_NEAREST" },
      { name: "LinearMipmapNearest", source: "ED_IMAGE_SAMPLING_LINEAR_MIPMAP_NEAREST" },
      { name: "LinearMipmapLinear", source: "ED_IMAGE_SAMPLING_LINEAR_MIPMAP_LINEAR" },
      { name: "CubicMitchell", source: "ED_IMAGE_SAMPLING_CUBIC_MITCHELL" },
      { name: "CubicCatmullRom", source: "ED_IMAGE_SAMPLING_CUBIC_CATMULL_ROM" },
      { name: "Anisotropic", source: "ED_IMAGE_SAMPLING_ANISOTROPIC" },
    ],
  },
  {
    name: "TextAlign",
    source: "ui",
    sourceEnum: "UiTextAlign",
    members: [
      { name: "Left", source: "UI_TEXT_ALIGN_LEFT" },
      { name: "Center", source: "UI_TEXT_ALIGN_CENTER" },
      { name: "Right", source: "UI_TEXT_ALIGN_RIGHT" },
    ],
  },
  {
    name: "TextVerticalAlign",
    source: "ui",
    sourceEnum: "UiTextVerticalAlign",
    members: [
      { name: "Top", source: "UI_TEXT_VERTICAL_ALIGN_TOP" },
      { name: "Center", source: "UI_TEXT_VERTICAL_ALIGN_CENTER" },
      { name: "Bottom", source: "UI_TEXT_VERTICAL_ALIGN_BOTTOM" },
    ],
  },
  {
    name: "TextOverflow",
    source: "ui",
    sourceEnum: "UiTextOverflow",
    members: [
      { name: "Clip", source: "UI_TEXT_OVERFLOW_CLIP" },
      { name: "Ellipsis", source: "UI_TEXT_OVERFLOW_ELLIPSIS" },
      { name: "Fade", source: "UI_TEXT_OVERFLOW_FADE" },
    ],
  },
  {
    name: "Orientation",
    source: "ui",
    sourceEnum: "UiOrientation",
    members: [
      { name: "None", source: "UI_ORIENTATION_NONE" },
      { name: "Horizontal", source: "UI_ORIENTATION_HORIZONTAL" },
      { name: "Vertical", source: "UI_ORIENTATION_VERTICAL" },
    ],
  },
  {
    name: "KeyEventType",
    source: "ui",
    sourceEnum: "UiKeyEventType",
    members: [
      { name: "Down", source: "UI_KEY_EVENT_DOWN" },
      { name: "Up", source: "UI_KEY_EVENT_UP" },
    ],
  },
  {
    name: "PointerEventType",
    source: "ui",
    sourceEnum: "UiEvent",
    members: [
      { name: "Down", source: "UI_EVENT_POINTER_DOWN" },
      { name: "Up", source: "UI_EVENT_POINTER_UP" },
      { name: "Move", source: "UI_EVENT_POINTER_MOVE" },
      { name: "Enter", source: "UI_EVENT_POINTER_ENTER" },
      { name: "Leave", source: "UI_EVENT_POINTER_LEAVE" },
      { name: "Cancel", source: "UI_EVENT_POINTER_CANCEL" },
    ],
  },
  {
    name: "KeyModifier",
    source: "ui",
    sourceEnum: "UiKeyModifier",
    members: [
      { name: "Shift", source: "UI_KEY_MOD_SHIFT" },
      { name: "Ctrl", source: "UI_KEY_MOD_CTRL" },
      { name: "Alt", source: "UI_KEY_MOD_ALT" },
      { name: "Meta", source: "UI_KEY_MOD_META" },
    ],
  },
  {
    name: "SemanticRole",
    source: "ui",
    sourceEnum: "UiSemanticRole",
    members: [
      { name: "None", source: "UI_SEMANTIC_NONE" },
      { name: "Button", source: "UI_SEMANTIC_BUTTON" },
      { name: "Textbox", source: "UI_SEMANTIC_TEXTBOX" },
      { name: "Link", source: "UI_SEMANTIC_LINK" },
      { name: "Heading", source: "UI_SEMANTIC_HEADING" },
      { name: "Form", source: "UI_SEMANTIC_FORM" },
      { name: "List", source: "UI_SEMANTIC_LIST" },
      { name: "ListItem", source: "UI_SEMANTIC_LIST_ITEM" },
      { name: "Image", source: "UI_SEMANTIC_IMAGE" },
      { name: "Dialog", source: "UI_SEMANTIC_DIALOG" },
      { name: "StaticText", source: "UI_SEMANTIC_STATIC_TEXT" },
      { name: "Checkbox", source: "UI_SEMANTIC_CHECKBOX" },
      { name: "Radio", source: "UI_SEMANTIC_RADIO" },
      { name: "RadioGroup", source: "UI_SEMANTIC_RADIO_GROUP" },
      { name: "Switch", source: "UI_SEMANTIC_SWITCH" },
      { name: "Slider", source: "UI_SEMANTIC_SLIDER" },
      { name: "ComboBox", source: "UI_SEMANTIC_COMBOBOX" },
    ],
  },
  {
    name: "PlatformFamily",
    source: "host",
    sourceEnum: "FuiPlatformFamily",
    members: [
      { name: "Unknown", source: "FUI_PLATFORM_UNKNOWN" },
      { name: "Apple", source: "FUI_PLATFORM_APPLE" },
      { name: "Windows", source: "FUI_PLATFORM_WINDOWS" },
      { name: "Linux", source: "FUI_PLATFORM_LINUX" },
    ],
  },
  {
    name: "HostEnvironment",
    source: "host",
    sourceEnum: "FuiHostEnvironment",
    members: [
      { name: "Unknown", source: "FUI_HOST_ENVIRONMENT_UNKNOWN" },
      { name: "Browser", source: "FUI_HOST_ENVIRONMENT_BROWSER" },
      { name: "Desktop", source: "FUI_HOST_ENVIRONMENT_DESKTOP" },
      { name: "Headless", source: "FUI_HOST_ENVIRONMENT_HEADLESS" },
    ],
  },
  {
    name: "HostCapability",
    source: "host",
    sourceEnum: "FuiHostCapability",
    members: [
      { name: "BrowserHistory", source: "FUI_HOST_CAPABILITY_BROWSER_HISTORY" },
      { name: "Reload", source: "FUI_HOST_CAPABILITY_RELOAD" },
      { name: "NewBrowsingContext", source: "FUI_HOST_CAPABILITY_NEW_BROWSING_CONTEXT" },
      { name: "OpenExternalUri", source: "FUI_HOST_CAPABILITY_OPEN_EXTERNAL_URI" },
      { name: "ClipboardRead", source: "FUI_HOST_CAPABILITY_CLIPBOARD_READ" },
      { name: "ClipboardWrite", source: "FUI_HOST_CAPABILITY_CLIPBOARD_WRITE" },
      { name: "FileDialogs", source: "FUI_HOST_CAPABILITY_FILE_DIALOGS" },
    ],
  },
  {
    name: "CursorStyle",
    source: "host",
    sourceEnum: "FuiCursorStyle",
    members: [
      { name: "Default", source: "FUI_CURSOR_DEFAULT" },
      { name: "Pointer", source: "FUI_CURSOR_POINTER" },
      { name: "Text", source: "FUI_CURSOR_TEXT" },
      { name: "Move", source: "FUI_CURSOR_MOVE" },
      { name: "Grab", source: "FUI_CURSOR_GRAB" },
      { name: "Grabbing", source: "FUI_CURSOR_GRABBING" },
      { name: "ResizeNS", source: "FUI_CURSOR_RESIZE_NS" },
      { name: "ResizeEW", source: "FUI_CURSOR_RESIZE_EW" },
    ],
  },
  {
    name: "SemanticCheckedState",
    source: "ui",
    sourceEnum: "UiSemanticCheckedState",
    members: [
      { name: "None", source: "UI_SEMANTIC_CHECKED_NONE" },
      { name: "False", source: "UI_SEMANTIC_CHECKED_FALSE" },
      { name: "True", source: "UI_SEMANTIC_CHECKED_TRUE" },
      { name: "Mixed", source: "UI_SEMANTIC_CHECKED_MIXED" },
    ],
  },
];
