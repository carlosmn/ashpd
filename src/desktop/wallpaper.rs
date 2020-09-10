//! # Examples
//!
//! Set a wallpaper from a URI:
//!
//! ```
//! use libportal::desktop::wallpaper::{
//!     WallpaperOptionsBuilder, WallpaperProxy, SetOn, WallpaperResponse
//! };
//! use libportal::{zbus, RequestProxy, WindowIdentifier};
//!
//! fn main() -> zbus::fdo::Result<()> {
//!     let connection = zbus::Connection::new_session()?;
//!     let proxy = WallpaperProxy::new(&connection)?;
//!
//!     let request_handle = proxy.set_wallpaper_uri(
//!         WindowIdentifier::default(),
//!         "file:///home/bilelmoussaoui/Downloads/adwaita-night.jpg",
//!         WallpaperOptionsBuilder::default()
//!             .show_preview(true)
//!             .set_on(SetOn::Both)
//!             .build(),
//!     )?;
//!
//!     let request = RequestProxy::new(&connection, &request_handle)?;
//!     request.on_response(|response: WallpaperResponse| {
//!         println!("{}", response.is_success() );
//!     })?;
//!     Ok(())
//! }
//! ```
use crate::{ResponseType, WindowIdentifier};
use serde::{self, Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use strum_macros::{AsRefStr, EnumString, IntoStaticStr, ToString};
use zbus::{dbus_proxy, fdo::Result};
use zvariant::{OwnedObjectPath, OwnedValue, Signature};
use zvariant_derive::{DeserializeDict, SerializeDict, Type, TypeDict};

#[derive(Deserialize, Debug, Clone, Copy, AsRefStr, EnumString, IntoStaticStr, ToString)]
#[serde(rename = "lowercase")]
/// Where to set the wallpaper on.
pub enum SetOn {
    /// Set the wallpaper only on the lockscreen.
    Lockscreen,
    /// Set the wallpaper only on the background.
    Background,
    /// Set the wallpaper on both lockscreen and background.
    Both,
}

impl zvariant::Type for SetOn {
    fn signature() -> Signature<'static> {
        Signature::from_string_unchecked("s".to_string())
    }
}

impl Serialize for SetOn {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        String::serialize(&self.to_string(), serializer)
    }
}

#[derive(SerializeDict, DeserializeDict, TypeDict, Debug, Default)]
/// Specified options for a set wallpaper request.
pub struct WallpaperOptions {
    /// Whether to show a preview of the picture
    /// Note that the portal may decide to show a preview even if this option is not set
    #[zvariant(rename = "show-preview")]
    pub show_preview: Option<bool>,
    /// Where to set the wallpaper on
    #[zvariant(rename = "set-on")]
    pub set_on: Option<SetOn>,
}

#[derive(Debug, Default)]
pub struct WallpaperOptionsBuilder {
    /// Whether to show a preview of the picture
    /// Note that the portal may decide to show a preview even if this option is not set
    pub show_preview: Option<bool>,
    /// Where to set the wallpaper on
    pub set_on: Option<SetOn>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct WallpaperResponse(pub ResponseType, pub HashMap<String, OwnedValue>);

impl WallpaperResponse {
    pub fn is_success(&self) -> bool {
        self.0 == ResponseType::Success
    }
}

impl WallpaperOptionsBuilder {
    pub fn show_preview(mut self, show_preview: bool) -> Self {
        self.show_preview = Some(show_preview);
        self
    }

    pub fn set_on(mut self, set_on: SetOn) -> Self {
        self.set_on = Some(set_on);
        self
    }

    pub fn build(self) -> WallpaperOptions {
        WallpaperOptions {
            set_on: self.set_on,
            show_preview: self.show_preview,
        }
    }
}

#[dbus_proxy(
    interface = "org.freedesktop.portal.Wallpaper",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
/// The interface lets sandboxed applications set the user's desktop background picture.
trait Wallpaper {
    /// Sets the lockscreen, background or both wallapers from a file descriptor
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `fd` - The wallapaper file description
    /// * `options` - A [`WallpaperOptions`]
    ///
    /// [`WallpaperOptions`]: ./struct.WallpaperOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    fn set_wallpaper_file(
        &self,
        parent_window: WindowIdentifier,
        fd: RawFd,
        options: WallpaperOptions,
    ) -> Result<OwnedObjectPath>;

    /// Sets the lockscreen, background or both wallapers from an URI
    ///
    /// Returns a [`Request`] handle
    ///
    /// # Arguments
    ///
    /// * `parent_window` - Identifier for the application window
    /// * `uri` - The wallapaper URI
    /// * `options` - A [`WallpaperOptions`]
    ///
    /// [`WallpaperOptions`]: ./struct.WallpaperOptions.html
    /// [`Request`]: ../request/struct.RequestProxy.html
    #[dbus_proxy(name = "SetWallpaperURI")]
    fn set_wallpaper_uri(
        &self,
        parent_window: WindowIdentifier,
        uri: &str,
        options: WallpaperOptions,
    ) -> Result<OwnedObjectPath>;

    /// version property
    #[dbus_proxy(property, name = "version")]
    fn version(&self) -> Result<u32>;
}
