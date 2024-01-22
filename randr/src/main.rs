use std::time::Duration;

use gnome_randr::display_config::OrgGnomeMutterDisplayConfig;

fn main() -> anyhow::Result<()> {
    let conn = dbus::blocking::Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.gnome.Mutter.DisplayConfig",
        "/org/gnome/Mutter/DisplayConfig",
        Duration::from_millis(5000),
    );

    let res = proxy.get_resources()?;
    println!("Outputs:");
    for out in res.outputs {
        println!("  {out}");
        if let Some(crtc) = res.crtcs.iter().find(|crtc| {
            std::convert::TryInto::<i32>::try_into(crtc.id).expect("CRTC id doesn't fit in i32")
                == out.crtc_id
        }) {
            println!("    CRTC: {crtc}");
            if let Some(mode) = res.modes.iter().find(|mode| {
                std::convert::TryInto::<i32>::try_into(mode.id).expect("Mode id doesn't fit in i32")
                    == crtc.mode_id
            }) {
                println!("    Mode: {mode}");
            }
        }
    }

    Ok(())
}
