use std::fmt::Display;

use dbus::arg;
use dbus::arg::RefArg;
use dbus::blocking;
use dbus_derive::DbusArgs;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

#[derive(FromPrimitive, ToPrimitive, Debug, Clone, Copy)]
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

/// A CRTC (CRT controller) is a logical monitor, ie a portion of the compositor coordinate space.
/// It might correspond to multiple monitors, when in clone mode, but not that
/// it is possible to implement clone mode also by setting different CRTCs to the same coordinates.
#[derive(Debug)]
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
    /// Other high-level properties that affect this CRTC; they are not necessarily reflected in the hardware.
    /// No property is specified in this version of the API
    _properties: arg::PropMap,
}

type CrtControllerTuple = (
    u32,
    i64,
    i32,
    i32,
    i32,
    i32,
    i32,
    u32,
    Vec<u32>,
    arg::PropMap,
);

impl From<CrtControllerTuple> for CrtController {
    fn from(val: CrtControllerTuple) -> Self {
        Self {
            id: val.0,
            winsys_id: val.1,
            x: val.2,
            y: val.3,
            width: val.4,
            height: val.5,
            mode_id: val.6,
            transform: FromPrimitive::from_u32(val.7)
                .expect("Recieved transform out of wayland spec"),
            transforms: val.8,
            _properties: val.9,
        }
    }
}

impl Display for CrtController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "id {}: ({},{}), {}x{}, {:?}",
            self.id, self.x, self.y, self.width, self.height, self.transform
        ))
    }
}

#[derive(Clone, Debug, DbusArgs)]
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
#[derive(Clone, Debug)]
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

type OutputTuple = (
    u32,
    i64,
    i32,
    Vec<u32>,
    String,
    Vec<u32>,
    Vec<u32>,
    arg::PropMap,
);

impl From<OutputTuple> for Output {
    fn from(val: OutputTuple) -> Self {
        Self {
            id: val.0,
            winsys_id: val.1,
            crtc_id: val.2,
            possible_crtc_ids: val.3,
            connector_name: val.4,
            mode_ids: val.5,
            clone_ids: val.6,
            props: val.7.into(),
        }
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("id {}: {}", self.id, self.props))
    }
}

#[derive(Clone, Debug)]
pub struct OutputChange {
    /// The API ID of the output to change
    pub id: u32,
    /// Properties whose value should be changed
    pub props: OutputProperties,
}

type OutputChangeTuple = (u32, arg::PropMap);

impl From<OutputChange> for OutputChangeTuple {
    fn from(val: OutputChange) -> Self {
        (val.id, val.props.into())
    }
}

/// Other high-level properties that affect this output; they are not necessarily reflected in the hardware.
#[derive(Default, Clone, Debug)]
pub struct OutputProperties {
    /// The human readable name of the manufacturer
    pub vendor: Option<String>,
    /// The human readable name
    pub product: Option<String>,
    /// The serial number of this particular hardware part
    pub serial: Option<String>,
    /// A human readable name of this output, to be shown in the UI
    pub display_name: Option<String>,
    /// The backlight value as a percentage (-1 if not supported)
    pub backlight: Option<i64>,
    /// Whether this output is primary or not
    pub primary: Option<bool>,
    /// Whether this output is for presentation only
    pub presentation: Option<bool>,
}

impl From<arg::PropMap> for OutputProperties {
    fn from(val: arg::PropMap) -> Self {
        Self {
            vendor: val
                .get("vendor")
                .and_then(|val| val.as_str().map(str::to_string)),
            product: val
                .get("product")
                .and_then(|val| val.as_str().map(str::to_string)),
            serial: val
                .get("serial")
                .and_then(|val| val.as_str().map(str::to_string)),
            display_name: val
                .get("display-name")
                .and_then(|val| val.as_str().map(str::to_string)),
            backlight: val.get("backlight").and_then(|val| val.as_i64()),
            primary: val
                .get("primary")
                .and_then(|val| val.as_i64().map(|num| num == 1)),
            presentation: val
                .get("presentation")
                .and_then(|val| val.as_i64().map(|num| num == 1)),
        }
    }
}

impl From<OutputProperties> for arg::PropMap {
    fn from(val: OutputProperties) -> Self {
        let mut props = arg::PropMap::new();
        val.vendor
            .map(|vendor| props.insert("vendor".to_string(), arg::Variant(Box::new(vendor))));
        val.product
            .map(|product| props.insert("product".to_string(), arg::Variant(Box::new(product))));
        val.serial
            .map(|serial| props.insert("serial".to_string(), arg::Variant(Box::new(serial))));
        val.display_name.map(|display_name| {
            props.insert(
                "display-name".to_string(),
                arg::Variant(Box::new(display_name)),
            )
        });
        val.backlight.map(|backlight| {
            props.insert("backlight".to_string(), arg::Variant(Box::new(backlight)))
        });
        val.primary
            .map(|primary| props.insert("primary".to_string(), arg::Variant(Box::new(primary))));
        val.presentation.map(|presentation| {
            props.insert(
                "presentation".to_string(),
                arg::Variant(Box::new(presentation)),
            )
        });
        props
    }
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
#[derive(Clone, Debug, DbusArgs)]
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
#[derive(Debug)]
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

type GetResourcesReturnTuple = (
    u32,
    Vec<CrtControllerTuple>,
    Vec<OutputTuple>,
    Vec<ModeTuple>,
    i32,
    i32,
);

impl From<GetResourcesReturnTuple> for GetResourcesReturn {
    fn from(val: GetResourcesReturnTuple) -> Self {
        GetResourcesReturn {
            serial: val.0,
            crtcs: val.1.into_iter().map(Into::into).collect(),
            outputs: val.2.into_iter().map(Into::into).collect(),
            modes: val.3.into_iter().map(Into::into).collect(),
            max_screen_width: val.4,
            max_screen_height: val.5,
        }
    }
}

#[derive(Clone, Debug)]
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

type ApplyConfigurationArgsTuple = (
    u32,
    bool,
    Vec<CrtControllerChangeTuple>,
    Vec<OutputChangeTuple>,
);

impl From<ApplyConfigurationArgs> for ApplyConfigurationArgsTuple {
    fn from(val: ApplyConfigurationArgs) -> Self {
        (
            val.serial,
            val.persistent,
            val.crtcs.into_iter().map(Into::into).collect(),
            val.outputs.into_iter().map(Into::into).collect(),
        )
    }
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
        let resources: GetResourcesReturnTuple =
            self.method_call("org.gnome.Mutter.DisplayConfig", "GetResources", ())?;
        Ok(resources.into())
    }

    fn apply_configuration(&self, args: ApplyConfigurationArgs) -> Result<(), dbus::Error> {
        let args: ApplyConfigurationArgsTuple = args.into();
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
