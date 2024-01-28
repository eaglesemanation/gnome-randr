use std::fmt::Display;

use dbus::blocking;
use dbus_derive::{DbusArgs, DbusEnum, DbusPropMap};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(DbusEnum, FromPrimitive, ToPrimitive, Debug, Clone, Copy)]
#[dbus_enum(as_type = "u32")]
pub enum Transform {
    Normal = 0,
    Normal90,
    Normal180,
    Normal270,
    Flipped,
    Flipped90,
    Flipped180,
    Flipped270,
}

impl From<Transform> for u32 {
    fn from(value: Transform) -> Self {
        value.to_u32().unwrap()
    }
}

impl TryFrom<u32> for Transform {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        <Self as FromPrimitive>::from_u32(value).ok_or("Transform u32 representation out of bound")
    }
}

/// A CRTC (CRT controller) is a logical monitor, ie a portion of the compositor coordinate space.
/// It might correspond to multiple monitors, when in clone mode, but not that
/// it is possible to implement clone mode also by setting different CRTCs to the same coordinates.
#[derive(DbusArgs, Debug)]
pub struct CrtController {
    /// The ID in the API of this CRTC
    pub id: u32,
    /// The low-level ID of this CRTC (which might be a XID, a KMS handle or something entirely different)
    pub winsys_id: i64,
    /// The geometry of this CRTC (might be invalid if the CRTC is not in use)
    pub x: i32,
    /// The geometry of this CRTC (might be invalid if the CRTC is not in use)
    pub y: i32,
    /// The geometry of this CRTC (might be invalid if the CRTC is not in use)
    pub width: i32,
    /// The geometry of this CRTC (might be invalid if the CRTC is not in use)
    pub height: i32,
    /// The current mode of the CRTC, or -1 if this CRTC is not used.
    /// Note: the size of the mode will always correspond to the width and height of the CRTC
    pub mode_id: i32,
    /// The current transform (exspressed according to the wayland protocol)
    pub transform: Transform,
    /// All posible transforms
    pub transforms: Vec<u32>,
    //FIXME: si.read().ok()? immediately returns None if I uncomment this, even though PropMap
    //should be present.
    //
    // Other high-level properties that affect this CRTC; they are not necessarily reflected in the hardware.
    // No property is specified in this version of the API
    //_properties: dbus::arg::PropMap,
}

impl Display for CrtController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "id {}: ({},{}), {}x{}, {:?}",
            self.id, self.x, self.y, self.width, self.height, self.transform
        ))
    }
}

#[derive(DbusArgs, Clone, Debug)]
pub struct CrtControllerChange {
    /// The API ID from the corresponding GetResources() call
    id: u32,
    /// The API ID of the new mode to configure the CRTC with, or -1 if the CRTC should be disabled
    mode_id: i32,
    /// The new coordinates of the top left corner.
    /// The geometry will be completed with the size information from new_mode.
    x: i32,
    /// The new coordinates of the top left corner.
    /// The geometry will be completed with the size information from new_mode.
    y: i32,
    /// The desired transform
    transform: u32,
    /// The API ID of outputs that should be assigned to this CRTC
    output_ids: Vec<u32>,
}

/// An output represents a physical screen, connected somewhere to the computer. Floating connectors are not exposed in the API.
#[derive(DbusArgs, Clone, Debug)]
pub struct Output {
    /// The ID in the API
    pub id: u32,
    /// The low-level ID of this output (XID or KMS handle)
    pub winsys_id: i64,
    /// The CRTC that is currently driving this output, or -1 if the output is disabled
    pub crtc_id: i32,
    /// All CRTCs that can control this output
    pub possible_crtc_ids: Vec<u32>,
    /// The name of the connector to which the output is attached (like VGA1 or HDMI)
    pub connector_name: String,
    /// Valid modes for this output
    pub mode_ids: Vec<u32>,
    /// Valid clones for this output, ie other outputs that can be assigned the same CRTC as this one;
    /// if you want to mirror two outputs that don't have each other in the clone list, you must configure two different CRTCs for the same geometry
    pub clone_ids: Vec<u32>,
    /// Other high-level properties that affect this output; they are not necessarily reflected in the hardware.
    pub props: OutputProperties,
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("id {}: {}", self.id, self.props))
    }
}

#[derive(DbusArgs, Clone, Debug)]
pub struct OutputChange {
    /// The API ID of the output to change
    pub id: u32,
    /// Properties whose value should be changed
    pub props: OutputProperties,
}

/// Other high-level properties that affect this output; they are not necessarily reflected in the hardware.
#[derive(DbusPropMap, Default, Clone, Debug)]
pub struct OutputProperties {
    /// The human readable name of the manufacturer
    pub vendor: Option<String>,
    /// The human readable name
    pub product: Option<String>,
    /// The serial number of this particular hardware part
    pub serial: Option<String>,
    /// A human readable name of this output, to be shown in the UI
    #[dbus_propmap(rename = "display-name")]
    pub display_name: Option<String>,
    /// The backlight value as a percentage (-1 if not supported)
    pub backlight: Option<i64>,
    /// Whether this output is primary or not
    pub primary: Option<bool>,
    /// Whether this output is for presentation only
    pub presentation: Option<bool>,
}

impl Display for OutputProperties {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} \"{} {}\"",
            self.display_name
                .clone()
                .unwrap_or("[Port name not found]".to_string()),
            self.vendor
                .clone()
                .unwrap_or("[Vendor not found]".to_string()),
            self.product
                .clone()
                .unwrap_or("[Display model not found]".to_string())
        ))?;
        if self.primary.is_some_and(|primary| primary) {
            f.write_str(" (primary)")?;
        }
        if self.presentation.is_some_and(|presentation| presentation) {
            f.write_str(" (presentation)")?;
        }
        Ok(())
    }
}

/// A mode represents a set of parameters that are applied to each output, such as resolution and refresh rate.
/// It is a separate object so that it can be referenced by CRTCs and outputs.
/// Multiple outputs in the same CRTCs must all have the same mode.
#[derive(DbusArgs, Clone, Debug)]
pub struct Mode {
    /// The ID in the API
    pub id: u32,
    /// The low-level ID of this mode
    pub winsys_id: i64,
    /// The resolution
    pub width: u32,
    /// The resolution
    pub height: u32,
    /// Refresh rate
    pub frequency: f64,
    /// Mode flags as defined in xf86drmMode.h and randr.h
    pub flags: u32,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "id: {}, {}x{}@{}",
            self.id,
            self.width,
            self.height,
            self.frequency.round()
        ))
    }
}

/// Current hardware layout
#[derive(DbusArgs, Debug)]
pub struct GetResourcesReturn {
    /// ID of current state of screen. Incremented by server to keep track of config changes
    pub serial: u32,
    /// Available CRTCs
    pub crtcs: Vec<CrtController>,
    /// Available outputs
    pub outputs: Vec<Output>,
    /// Available modes
    pub modes: Vec<Mode>,
    pub max_screen_width: i32,
    pub max_screen_height: i32,
}

#[derive(DbusArgs, Clone, Debug)]
pub struct ApplyConfigurationArgs {
    serial: u32,
    persistent: bool,
    /// crtcs represents the new logical configuration, as a list of structures.
    /// Note: CRTCs not referenced in the array will be disabled.
    crtcs: Vec<CrtControllerChange>,
    /// outputs represent the output property changes.
    /// Note: both for CRTCs and outputs, properties not included in the dictionary will not be changed.
    ///
    /// Note: unrecognized properties will have no effect, but if the configuration change succeeds
    /// the property will be reported by the next GetResources() call, and if @persistent is true, it will also be saved to disk.
    outputs: Vec<OutputChange>,
}

pub trait OrgGnomeMutterDisplayConfig {
    fn get_resources(&self) -> Result<GetResourcesReturn, dbus::Error>;
    fn apply_configuration(&self, args: ApplyConfigurationArgs) -> Result<(), dbus::Error>;
    fn change_backlight(&self, serial: u32, output: u32, value: i32) -> Result<(), dbus::Error>;
    fn get_crtc_gamma(
        &self,
        serial: u32,
        crtc: u32,
    ) -> Result<(Vec<u16>, Vec<u16>, Vec<u16>), dbus::Error>;
    fn set_crtc_gamma(
        &self,
        serial: u32,
        crtc: u32,
        red: Vec<u16>,
        green: Vec<u16>,
        blue: Vec<u16>,
    ) -> Result<(), dbus::Error>;
    fn power_save_mode(&self) -> Result<i32, dbus::Error>;
    fn set_power_save_mode(&self, value: i32) -> Result<(), dbus::Error>;
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgGnomeMutterDisplayConfig
    for blocking::Proxy<'a, C>
{
    fn get_resources(&self) -> Result<GetResourcesReturn, dbus::Error> {
        self.method_call("org.gnome.Mutter.DisplayConfig", "GetResources", ())
    }

    fn apply_configuration(&self, args: ApplyConfigurationArgs) -> Result<(), dbus::Error> {
        self.method_call("org.gnome.Mutter.DisplayConfig", "ApplyConfiguration", args)
    }

    fn change_backlight(&self, serial: u32, output: u32, value: i32) -> Result<(), dbus::Error> {
        self.method_call(
            "org.gnome.Mutter.DisplayConfig",
            "ChangeBacklight",
            (serial, output, value),
        )
    }

    fn get_crtc_gamma(
        &self,
        serial: u32,
        crtc_id: u32,
    ) -> Result<(Vec<u16>, Vec<u16>, Vec<u16>), dbus::Error> {
        self.method_call(
            "org.gnome.Mutter.DisplayConfig",
            "GetCrtcGamma",
            (serial, crtc_id),
        )
    }

    fn set_crtc_gamma(
        &self,
        serial: u32,
        crtc: u32,
        red: Vec<u16>,
        green: Vec<u16>,
        blue: Vec<u16>,
    ) -> Result<(), dbus::Error> {
        self.method_call(
            "org.gnome.Mutter.DisplayConfig",
            "SetCrtcGamma",
            (serial, crtc, red, green, blue),
        )
    }

    fn power_save_mode(&self) -> Result<i32, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.gnome.Mutter.DisplayConfig",
            "PowerSaveMode",
        )
    }

    fn set_power_save_mode(&self, value: i32) -> Result<(), dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.gnome.Mutter.DisplayConfig",
            "PowerSaveMode",
            value,
        )
    }
}
