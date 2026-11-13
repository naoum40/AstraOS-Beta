// ==============================================================================
// AstraOS Dock — lib.rs (bibliothèque pour intégration astra-shell)
// ==============================================================================
pub mod magnification;
pub mod dock_manager;
pub mod icon_manager;
pub mod visibility;
pub mod intellihide;
pub mod signal_manager;
pub mod window_preview;
pub mod media_manager;
pub mod prefs;

// Re-export les types principaux
pub use dock_manager::{DockManager, DockConfig, DockPosition};
pub use magnification::Magnification;
pub use icon_manager::{IconManager, IconConfig, IndicatorStyle};
pub use visibility::{DockVisibility, VisibilityConfig, DockEdge};
pub use intellihide::Intellihide;
pub use signal_manager::SignalManager;
pub use window_preview::WindowPreviewPopup;
pub use media_manager::MediaManager;
