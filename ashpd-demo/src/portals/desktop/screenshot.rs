use ashpd::{desktop::screenshot, WindowIdentifier};
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

use crate::widgets::ColorWidget;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/screenshot.ui")]
    pub struct ScreenshotPage {
        #[template_child]
        pub interactive_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub modal_switch: TemplateChild<gtk::Switch>,
        #[template_child]
        pub color_widget: TemplateChild<ColorWidget>,
        #[template_child]
        pub screenshot_photo: TemplateChild<gtk::Picture>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ScreenshotPage {
        const NAME: &'static str = "ScreenshotPage";
        type Type = super::ScreenshotPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.set_layout_manager_type::<adw::ClampLayout>();
            klass.install_action(
                "screenshot.pick-color",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        page.pick_color().await;
                    }));
                },
            );
            klass.install_action(
                "screenshot.screenshot",
                None,
                move |page, _action, _target| {
                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(@weak page => async move {
                        page.screenshot().await;
                    }));
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for ScreenshotPage {
        fn constructed(&self, _obj: &Self::Type) {
            self.screenshot_photo.set_overflow(gtk::Overflow::Hidden);
        }
    }
    impl WidgetImpl for ScreenshotPage {}
    impl BinImpl for ScreenshotPage {}
}

glib::wrapper! {
    pub struct ScreenshotPage(ObjectSubclass<imp::ScreenshotPage>) @extends gtk::Widget, adw::Bin;
}

impl ScreenshotPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a ScreenshotPage")
    }

    async fn pick_color(&self) {
        // used for retrieving a window identifier
        let root = self.native().unwrap();
        let self_ = imp::ScreenshotPage::from_instance(self);
        let identifier = WindowIdentifier::from_native(&root).await;
        if let Ok(color) = screenshot::pick_color(&identifier).await {
            self_.color_widget.set_rgba(color.into());
        }
    }

    async fn screenshot(&self) {
        let self_ = imp::ScreenshotPage::from_instance(self);
        // used for retrieving a window identifier
        let root = self.native().unwrap();
        let identifier = WindowIdentifier::from_native(&root).await;

        let interactive = self_.interactive_switch.is_active();
        let modal = self_.modal_switch.is_active();

        if let Ok(uri) = screenshot::take(&identifier, interactive, modal).await {
            let file = gio::File::for_uri(&uri);
            self_.screenshot_photo.set_file(Some(&file));
            self_.revealer.show(); // Revealer has a weird issue where it still
                                   // takes space even if it's child is hidden

            self_.revealer.set_reveal_child(true);
        }
    }
}
