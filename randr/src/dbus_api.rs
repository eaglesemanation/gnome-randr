use std::{fmt::Display, time::Duration};

use dbus::blocking;
use dbus_derive::{DbusArgs, DbusEnum, DbusPropMap, DbusStruct};
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
/// It might correspond to multiple monitors, when in clone mode, but note that
/// it is possible to implement clone mode also by setting different CRTCs to the same coordinates.
#[derive(DbusStruct, Clone, Debug)]
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

#[derive(DbusStruct, Clone, Debug)]
pub struct CrtControllerChange {
    /// The API ID from the corresponding GetResources() call
    pub id: u32,
    /// The API ID of the new mode to configure the CRTC with, or -1 if the CRTC should be disabled
    pub mode_id: i32,
    /// The new coordinates of the top left corner.
    /// The geometry will be completed with the size information from new_mode.
    pub x: i32,
    /// The new coordinates of the top left corner.
    /// The geometry will be completed with the size information from new_mode.
    pub y: i32,
    /// The desired transform
    pub transform: u32,
    /// The API ID of outputs that should be assigned to this CRTC
    pub output_ids: Vec<u32>,
}

/// An output represents a physical screen, connected somewhere to the computer. Floating connectors are not exposed in the API.
#[derive(DbusStruct, Clone, Debug)]
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

#[derive(DbusStruct, Clone, Debug)]
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
#[derive(DbusStruct, Clone, Debug)]
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

/// Current hardware layout
#[derive(DbusArgs, Clone, Debug)]
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
    pub serial: u32,
    pub persistent: bool,
    /// crtcs represents the new logical configuration, as a list of structures.
    /// Note: CRTCs not referenced in the array will be disabled.
    pub crtcs: Vec<CrtControllerChange>,
    /// outputs represent the output property changes.
    /// Note: both for CRTCs and outputs, properties not included in the dictionary will not be changed.
    ///
    /// Note: unrecognized properties will have no effect, but if the configuration change succeeds
    /// the property will be reported by the next GetResources() call, and if @persistent is true, it will also be saved to disk.
    pub outputs: Vec<OutputChange>,
}

#[derive(DbusArgs, Clone, Debug)]
pub struct ChangeBacklightArgs {
    pub serial: u32,
    /// the API id of the output
    pub output: u32,
    /// the new backlight value
    pub value: i32,
}

#[derive(DbusArgs, Clone, Debug)]
pub struct CrtcGamma {
    /// red gamma ramp
    pub red: Vec<u16>,
    /// green gamma ramp
    pub green: Vec<u16>,
    /// blue gamma ramp
    pub blue: Vec<u16>,
}

pub struct OrgGnomeMutterDisplayConfig<'a, C> {
    proxy: blocking::Proxy<'a, C>,
}

pub type DisplayConfig<'a, 'b> = OrgGnomeMutterDisplayConfig<'a, &'b blocking::Connection>;

impl<'a> DisplayConfig<'_, 'a> {
    pub fn new(conn: &'a blocking::Connection) -> Self {
        let proxy = blocking::Proxy::new(
            "org.gnome.Mutter.DisplayConfig",
            "/org/gnome/Mutter/DisplayConfig",
            Duration::from_millis(5000),
            conn,
        );
        Self { proxy }
    }

    pub fn get_resources(&self) -> Result<GetResourcesReturn, dbus::Error> {
        self.proxy
            .method_call("org.gnome.Mutter.DisplayConfig", "GetResources", ())
    }

    pub fn apply_configuration(&self, args: ApplyConfigurationArgs) -> Result<(), dbus::Error> {
        self.proxy
            .method_call("org.gnome.Mutter.DisplayConfig", "ApplyConfiguration", args)
    }

    pub fn change_backlight(&self, args: ChangeBacklightArgs) -> Result<(), dbus::Error> {
        self.proxy
            .method_call("org.gnome.Mutter.DisplayConfig", "ChangeBacklight", args)
    }

    pub fn get_crtc_gamma(&self, serial: u32, crtc: u32) -> Result<CrtcGamma, dbus::Error> {
        self.proxy.method_call(
            "org.gnome.Mutter.DisplayConfig",
            "GetCrtcGamma",
            (serial, crtc),
        )
    }

    pub fn set_crtc_gamma(
        &self,
        serial: u32,
        crtc: u32,
        red: Vec<u16>,
        green: Vec<u16>,
        blue: Vec<u16>,
    ) -> Result<(), dbus::Error> {
        self.proxy.method_call(
            "org.gnome.Mutter.DisplayConfig",
            "SetCrtcGamma",
            (serial, crtc, red, green, blue),
        )
    }

    pub fn power_save_mode(&self) -> Result<i32, dbus::Error> {
        blocking::stdintf::org_freedesktop_dbus::Properties::get(
            &self.proxy,
            "org.gnome.Mutter.DisplayConfig",
            "PowerSaveMode",
        )
    }

    pub fn set_power_save_mode(&self, value: i32) -> Result<(), dbus::Error> {
        blocking::stdintf::org_freedesktop_dbus::Properties::set(
            &self.proxy,
            "org.gnome.Mutter.DisplayConfig",
            "PowerSaveMode",
            value,
        )
    }
}
