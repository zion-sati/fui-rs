use fui::platform::{HostContext, HostEnvironment, PlatformFamily};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoCapability {
    BrowserRouting,
    ExternalFileDrop,
    FileDialogs,
    HostWorkers,
    NativeContextMenus,
    PageZoom,
    PersistedNavigation,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DemoCapabilities(u32);

impl DemoCapabilities {
    const BROWSER_ROUTING: u32 = 1 << 0;
    const EXTERNAL_FILE_DROP: u32 = 1 << 1;
    const FILE_DIALOGS: u32 = 1 << 2;
    const HOST_WORKERS: u32 = 1 << 3;
    const NATIVE_CONTEXT_MENUS: u32 = 1 << 4;
    const PAGE_ZOOM: u32 = 1 << 5;
    const PERSISTED_NAVIGATION: u32 = 1 << 6;

    pub const fn browser() -> Self {
        Self(
            Self::BROWSER_ROUTING
                | Self::EXTERNAL_FILE_DROP
                | Self::FILE_DIALOGS
                | Self::HOST_WORKERS
                | Self::PAGE_ZOOM
                | Self::PERSISTED_NAVIGATION,
        )
    }

    pub const fn native() -> Self {
        Self(
            Self::EXTERNAL_FILE_DROP
                | Self::FILE_DIALOGS
                | Self::HOST_WORKERS
                | Self::NATIVE_CONTEXT_MENUS
                | Self::PAGE_ZOOM,
        )
    }

    pub const fn supports(self, capability: DemoCapability) -> bool {
        let flag = match capability {
            DemoCapability::BrowserRouting => Self::BROWSER_ROUTING,
            DemoCapability::ExternalFileDrop => Self::EXTERNAL_FILE_DROP,
            DemoCapability::FileDialogs => Self::FILE_DIALOGS,
            DemoCapability::HostWorkers => Self::HOST_WORKERS,
            DemoCapability::NativeContextMenus => Self::NATIVE_CONTEXT_MENUS,
            DemoCapability::PageZoom => Self::PAGE_ZOOM,
            DemoCapability::PersistedNavigation => Self::PERSISTED_NAVIGATION,
        };
        self.0 & flag != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoLinks {
    pub source_root: String,
    pub documentation_root: String,
}

impl DemoLinks {
    pub fn new(source_root: impl Into<String>, documentation_root: impl Into<String>) -> Self {
        Self {
            source_root: source_root.into().trim_end_matches('/').to_string(),
            documentation_root: documentation_root.into().trim_end_matches('/').to_string(),
        }
    }

    pub fn source(&self, relative_path: &str) -> String {
        format!(
            "{}/{}",
            self.source_root,
            relative_path.trim_start_matches('/')
        )
    }

    pub fn documentation(&self, relative_path: &str) -> String {
        format!(
            "{}/{}",
            self.documentation_root,
            relative_path.trim_start_matches('/')
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoTerminology {
    pub primary_modifier: &'static str,
    pub application_kind: &'static str,
}

impl DemoTerminology {
    fn for_host(host: HostContext) -> Self {
        Self {
            primary_modifier: if host.platform_family == PlatformFamily::Apple {
                "Command"
            } else {
                "Ctrl"
            },
            application_kind: if host.environment == HostEnvironment::Browser {
                "web application"
            } else {
                "desktop application"
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoEnvironment {
    host: HostContext,
    capabilities: DemoCapabilities,
    links: DemoLinks,
    terminology: DemoTerminology,
}

impl DemoEnvironment {
    pub fn new(host: HostContext, capabilities: DemoCapabilities, links: DemoLinks) -> Self {
        Self {
            host,
            capabilities,
            links,
            terminology: DemoTerminology::for_host(host),
        }
    }

    pub fn browser(platform: PlatformFamily, links: DemoLinks) -> Self {
        Self::new(
            HostContext::new(platform, HostEnvironment::Browser, 0),
            DemoCapabilities::browser(),
            links,
        )
    }

    pub fn native(platform: PlatformFamily, links: DemoLinks) -> Self {
        Self::new(
            HostContext::new(platform, HostEnvironment::Desktop, 0),
            DemoCapabilities::native(),
            links,
        )
    }

    pub fn host(&self) -> HostContext {
        self.host
    }

    pub fn capabilities(&self) -> DemoCapabilities {
        self.capabilities
    }

    pub fn supports(&self, capability: DemoCapability) -> bool {
        self.capabilities.supports(capability)
    }

    pub fn links(&self) -> &DemoLinks {
        &self.links
    }

    pub fn terminology(&self) -> &DemoTerminology {
        &self.terminology
    }
}
