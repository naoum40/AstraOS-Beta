// sidebar.rs - Astra VM left navigation rail (280px).
//
// Vertical `gtk4::Box` hosting:
//
//   1. Section header "Mes VMs".
//   2. Scrollable list of `VmItem` rows (one per VM).
//   3. Section header "Récent" (placeholder, empty for now — subagent D
//      will populate from the recent-VMs list).
//
// State held:
//   - `items`: a `Rc<RefCell<Vec<VmItem>>>` snapshot of the rendered
//     items (rebuilt by `set_vms`).
//   - `selected_id`: id of the currently highlighted VM, or `None`.
//   - `on_select`: callback fired when the user clicks a VM row —
//     receives the VM id (never `None`, since clicking a row always
//     selects it).
//
// Selection routing uses the same `Rc<RefCell<Option<Box<dyn Fn>>>`
// pattern as `astra-pass/src/ui/sidebar.rs` — keeps the sidebar
// decoupled from the main area.

use crate::config::VmConfig;
use crate::ui::vm_item::VmItem;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, PolicyType, ScrolledWindow};
use std::cell::RefCell;
use std::rc::Rc;

/// The sidebar widget.
#[derive(Clone)]
pub struct Sidebar {
    widget: GtkBox,
    /// Container holding the rendered VM items.
    list_box: GtkBox,
    /// Snapshot of the rendered VM items.
    items: Rc<RefCell<Vec<VmItem>>>,
    /// Id of the currently active VM (or `None`).
    selected_id: Rc<RefCell<Option<String>>>,
    /// Selection callback — fires with the clicked VM's id.
    on_select: Rc<RefCell<Option<Box<dyn Fn(&str)>>>>,
}

impl Sidebar {
    /// Build the sidebar. Empty until `set_vms` is called.
    pub fn new() -> Self {
        let widget = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .css_classes(["sidebar"])
            .width_request(280)
            .build();

        // --- Section header: "Mes VMs" -------------------------------
        let section_title = Label::builder()
            .label("Mes VMs")
            .halign(gtk4::Align::Start)
            .margin_start(16)
            .margin_top(16)
            .margin_bottom(8)
            .css_classes(["section-title"])
            .build();
        widget.append(&section_title);

        // --- Scrollable list of VM items -----------------------------
        let list_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .css_classes(["vm-list"])
            .hexpand(true)
            .halign(gtk4::Align::Fill)
            .build();

        let scroll = ScrolledWindow::builder()
            .child(&list_box)
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .hexpand(true)
            .vexpand(true)
            .css_classes(["vm-list-scroll"])
            .build();
        widget.append(&scroll);

        // --- Section header: "Récent" (placeholder) -----------------
        let recent_title = Label::builder()
            .label("R\u{E9}cent") // Récent
            .halign(gtk4::Align::Start)
            .margin_start(16)
            .margin_top(16)
            .margin_bottom(8)
            .css_classes(["section-title"])
            .build();
        widget.append(&recent_title);

        let recent_empty = Label::builder()
            .label("<span alpha=\"55%\">Aucune VM r\u{E9}cente</span>") // Aucune VM récente
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .margin_start(16)
            .margin_bottom(12)
            .css_classes(["sidebar-hint"])
            .build();
        widget.append(&recent_empty);

        let items: Rc<RefCell<Vec<VmItem>>> = Rc::new(RefCell::new(Vec::new()));
        let selected_id: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let on_select: Rc<RefCell<Option<Box<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));

        // Empty-state placeholder — shown when there are no VMs.
        let empty = Label::builder()
            .label("<span alpha=\"55%\">Aucune VM configur\u{E9}e.\nCliquez sur \u{201C}+ Nouvelle VM\u{201D}.</span>") // Aucune VM configurée. Cliquez sur "+ Nouvelle VM".
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .margin_start(16)
            .margin_top(12)
            .css_classes(["sidebar-empty"])
            .build();
        list_box.append(&empty);

        Self {
            widget,
            list_box,
            items,
            selected_id,
            on_select,
        }
    }

    /// Replace the rendered VM list. Rebuilds the list box from scratch
    /// (fine for the typical < 50-VM config). Selection is cleared.
    pub fn set_vms(&self, vms: &[VmConfig]) {
        // Clear existing children (item rows + the empty placeholder).
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        self.items.borrow_mut().clear();
        *self.selected_id.borrow_mut() = None;

        if vms.is_empty() {
            // Empty-state placeholder.
            let empty = Label::builder()
                .label("<span alpha=\"55%\">Aucune VM configur\u{E9}e.\nCliquez sur \u{201C}+ Nouvelle VM\u{201D}.</span>")
                .use_markup(true)
                .halign(gtk4::Align::Start)
                .margin_start(16)
                .margin_top(12)
                .css_classes(["sidebar-empty"])
                .build();
            self.list_box.append(&empty);
            return;
        }

        let sidebar_clone = self.clone();
        for vm in vms {
            let item = VmItem::new(vm);
            let id = vm.id.clone();

            // Wire the click: forward to `select_vm`.
            let sc = sidebar_clone.clone();
            let clicked_id = id.clone();
            item.widget().connect_clicked(move |_| {
                sc.select_vm(&clicked_id);
            });

            self.list_box.append(item.widget());
            self.items.borrow_mut().push(item);
        }
    }

    /// Select a VM by id. Updates the `active` CSS class on every item
    /// + fires the registered `on_select` callback.
    pub fn select_vm(&self, id: &str) {
        *self.selected_id.borrow_mut() = Some(id.to_string());
        for item in self.items.borrow().iter() {
            item.set_active(item.vm_id() == id);
        }
        if let Some(cb) = self.on_select.borrow().as_ref() {
            cb(id);
        }
    }

    /// Update the running-state indicator on the VM item with the
    /// given id. No-op if the id isn't currently rendered.
    pub fn set_vm_running(&self, id: &str, running: bool) {
        for item in self.items.borrow().iter() {
            if item.vm_id() == id {
                item.set_running(running);
                break;
            }
        }
    }

    /// Install the selection callback. Fires with the VM id of the
    /// clicked row (never `None`).
    pub fn set_on_select<F: Fn(&str) + 'static>(&self, f: F) {
        *self.on_select.borrow_mut() = Some(Box::new(f));
    }

    /// Borrow the underlying GTK widget for layout parenting.
    pub fn widget(&self) -> &GtkBox {
        &self.widget
    }
}
