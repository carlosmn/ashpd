use std::os::unix::prelude::RawFd;

use ashpd::{desktop::camera, zbus};
use glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::widgets::CameraPaintable;

mod imp {
    use adw::subclass::prelude::*;
    use gtk::CompositeTemplate;

    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/com/belmoussaoui/ashpd/demo/camera.ui")]
    pub struct CameraPage {
        #[template_child]
        pub camera_available: TemplateChild<gtk::Label>,
        #[template_child]
        pub picture: TemplateChild<gtk::Picture>,
        pub paintable: CameraPaintable,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CameraPage {
        const NAME: &'static str = "CameraPage";
        type Type = super::CameraPage;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            klass.install_action("camera.start", None, move |page, _action, _target| {
                let ctx = glib::MainContext::default();
                ctx.spawn_local(clone!(@weak page => async move {
                    page.start_stream().await;
                }));
            });
            klass.install_action("camera.stop", None, move |page, _, _| {
                page.stop_stream();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    impl ObjectImpl for CameraPage {
        fn constructed(&self, obj: &Self::Type) {
            self.picture.set_paintable(Some(&self.paintable));
            obj.action_set_enabled("camera.stop", false);
            self.parent_constructed(obj);
        }
    }
    impl WidgetImpl for CameraPage {}
    impl BinImpl for CameraPage {}
}

glib::wrapper! {
    pub struct CameraPage(ObjectSubclass<imp::CameraPage>) @extends gtk::Widget, adw::Bin;
}

impl CameraPage {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create a CameraPage")
    }

    async fn start_stream(&self) {
        let self_ = imp::CameraPage::from_instance(&self);

        self.action_set_enabled("camera.stop", true);
        self.action_set_enabled("camera.start", false);
        match stream().await {
            Ok(Some(stream_fd)) => {
                self_.paintable.set_pipewire_fd(stream_fd);
                self_.revealer.set_reveal_child(true);
                self_.camera_available.set_text("Yes");
            }
            Ok(None) => {
                self_.camera_available.set_text("No");
            }
            Err(err) => {
                tracing::error!("Failed to start a camera stream {:#?}", err);
                self.stop_stream();
            }
        }
    }

    fn stop_stream(&self) {
        let self_ = imp::CameraPage::from_instance(self);
        self.action_set_enabled("camera.stop", false);
        self.action_set_enabled("camera.start", true);

        self_.paintable.close_pipeline();
        self_.revealer.set_reveal_child(false);
    }
}

async fn stream() -> ashpd::Result<Option<RawFd>> {
    let connection = zbus::azync::Connection::session().await?;
    let proxy = camera::CameraProxy::new(&connection).await?;
    if proxy.is_camera_present().await? {
        proxy.access_camera().await?;

        Ok(Some(proxy.open_pipe_wire_remote().await?))
    } else {
        Ok(None)
    }
}
