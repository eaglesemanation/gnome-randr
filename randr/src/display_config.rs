use std::{convert::Infallible, error::Error, fmt::Display};

use dbus::arg;
use dbus::arg::RefArg;
use dbus::blocking;
use dbus_derive::DbusArgs;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

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

impl dbus_traits::DbusArg<u8> for Transform {
    type Error = &'static str;
    fn dbus_arg_try_from(self) -> Result<u8, Self::Error> {
        Ok(self.to_u8().unwrap())
    }
    fn dbus_arg_try_into(value: u8) -> Result<Self, Self::Error> {
        FromPrimitive::from_u8(value).ok_or("Could not parse transform")
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
    #[dbus_arg(target_type = "u8")]
    pub transform: Transform,
    /// All posible transforms
    pub transforms: Vec<u32>,
    /// Other high-level properties that affect this CRTC; they are not necessarily reflected in the hardware.
    /// No property is specified in this version of the API
    _properties: arg::PropMap,
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
    #[dbus_arg(target_type = "arg::PropMap")]
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
    #[dbus_arg(target_type = "arg::PropMap")]
    pub props: OutputProperties,
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

impl dbus_traits::DbusArg<arg::PropMap> for OutputProperties {
    type Error = Infallible;

    fn dbus_arg_try_from(self) -> Result<arg::PropMap, Self::Error> {
        let mut props = arg::PropMap::new();
        self.vendor
            .map(|vendor| props.insert("vendor".to_string(), arg::Variant(Box::new(vendor))));
        self.product
            .map(|product| props.insert("product".to_string(), arg::Variant(Box::new(product))));
        self.serial
            .map(|serial| props.insert("serial".to_string(), arg::Variant(Box::new(serial))));
        self.display_name.map(|display_name| {
            props.insert(
                "display-name".to_string(),
                arg::Variant(Box::new(display_name)),
            )
        });
        self.backlight.map(|backlight| {
            props.insert("backlight".to_string(), arg::Variant(Box::new(backlight)))
        });
        self.primary
            .map(|primary| props.insert("primary".to_string(), arg::Variant(Box::new(primary))));
        self.presentation.map(|presentation| {
            props.insert(
                "presentation".to_string(),
                arg::Variant(Box::new(presentation)),
            )
        });
        Ok(props)
    }

    fn dbus_arg_try_into(value: arg::PropMap) -> Result<Self, Self::Error> {
        Ok(Self {
            vendor: value
                .get("vendor")
                .and_then(|val| val.as_str().map(str::to_string)),
            product: value
                .get("product")
                .and_then(|val| val.as_str().map(str::to_string)),
            serial: value
                .get("serial")
                .and_then(|val| val.as_str().map(str::to_string)),
            display_name: value
                .get("display-name")
                .and_then(|val| val.as_str().map(str::to_string)),
            backlight: value.get("backlight").and_then(|val| val.as_i64()),
            primary: value
                .get("primary")
                .and_then(|val| val.as_i64().map(|num| num == 1)),
            presentation: value
                .get("presentation")
                .and_then(|val| val.as_i64().map(|num| num == 1)),
        })
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
    #[dbus_arg(derived)]
    pub crtcs: Vec<CrtController>,
    /// Available outputs
    #[dbus_arg(derived)]
    pub outputs: Vec<Output>,
    /// Available modes
    #[dbus_arg(derived)]
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
    #[dbus_arg(derived)]
    crtcs: Vec<CrtControllerChange>,
    /// outputs represent the output property changes.
    /// Note: both for CRTCs and outputs, properties not included in the dictionary will not be changed.
    ///
    /// Note: unrecognized properties will have no effect, but if the configuration change succeeds
    /// the property will be reported by the next GetResources() call, and if @persistent is true, it will also be saved to disk.
    #[dbus_arg(derived)]
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
        let resources: GetResourcesReturnTuple =
            self.method_call("org.gnome.Mutter.DisplayConfig", "GetResources", ())?;
        resources.try_into().map_err(|err: Box<dyn Error>| {
            dbus::Error::new_custom("InvalidReply", &err.to_string())
        })
    }

    fn apply_configuration(&self, args: ApplyConfigurationArgs) -> Result<(), dbus::Error> {
        let args: ApplyConfigurationArgsTuple =
            args.try_into().map_err(|err: Box<dyn Error>| {
                dbus::Error::new_custom("InvalidReply", &err.to_string())
            })?;
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
