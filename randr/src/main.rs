use gnome_randr::{cli::Cli, dbus_api::DisplayConfig, mode_db::ModeDb, output::Output};

fn main() -> anyhow::Result<()> {
    let args = Cli::parse_from_env()?;

    let conn = dbus::blocking::Connection::new_session()?;
    let display_config = DisplayConfig::new(&conn);

    let resources = display_config.get_resources()?;
    let mode_db = ModeDb::new(&resources.modes);

    if args.outputs.is_empty() {
        todo!("Convert dbus return into useful outputs struct");
        //display_outputs(args, resources)?;
    } else {
        todo!("Actually modify config");
    }

    Ok(())
}

fn display_outputs(args: Cli, outputs: &[Output]) -> anyhow::Result<()> {
    /*
    for out in res.outputs {
        let unique_supported_modes = mode_db.get_modes_by_ids(&out.mode_ids);
        let grouped_modes = mode_db::group_modes_by_res(&unique_supported_modes);
        if let Some(crtc) = res.crtcs.iter().find(|crtc| {
            std::convert::TryInto::<i32>::try_into(crtc.id).expect("CRTC id doesn't fit in i32")
                == out.crtc_id
        }) {
            if let Some(mode) =
                mode_db.get_mode_by_id(crtc.mode_id.try_into().expect("Mode id doesn't fit in i32"))
            {
            }
        }
    }
    */

    Ok(())
}
