use anyhow::anyhow;
use derive_builder::Builder;
use lexopt::ValueExt;

use crate::mode_db;

#[derive(Debug)]
pub struct Cli {
    pub outputs: Vec<OutputArgs>,
}

impl Cli {
    pub fn parse(mut p: lexopt::Parser) -> anyhow::Result<Self> {
        let mut outputs = vec![];

        while let Some(arg) = p.next()? {
            use lexopt::prelude::*;
            match arg {
                Long("help") => {
                    println!("Usage: gnome-randr [--output <OUTPUT> [--resolution <WIDTH>x<HEIGHT>] [--fps <FPS>] [--auto] [--off]]")
                }
                Long("output") => {
                    outputs = OutputArgs::parse(&mut p)?;
                    break;
                }
                _ => return Err(arg.unexpected().into()),
            }
        }

        Ok(Self { outputs })
    }

    pub fn parse_from_env() -> anyhow::Result<Self> {
        Self::parse(lexopt::Parser::from_env())
    }
}

#[derive(Debug, PartialEq, Eq, Builder)]
pub struct OutputArgs {
    #[builder(setter(into))]
    pub name: String,
    #[builder(default)]
    pub auto: bool,
    #[builder(default)]
    pub off: bool,
    #[builder(setter(strip_option), default)]
    pub resolution: Option<mode_db::Resolution>,
    #[builder(setter(strip_option), default)]
    pub framerate: Option<u32>,
}

impl OutputArgs {
    fn parse(p: &mut lexopt::Parser) -> anyhow::Result<Vec<Self>> {
        let mut outputs = vec![];
        loop {
            let name: String = p.value()?.parse()?;
            if outputs.iter().any(|o: &OutputArgs| o.name == name) {
                return Err(anyhow!("--output {name} is duplicated"));
            }
            let mut next_output = false;

            let mut output_builder = OutputArgsBuilder::default();
            output_builder.name(name.clone());

            while let Some(arg) = p.next()? {
                use lexopt::prelude::*;
                // Create arg string representation for error messaging
                let arg_str = match arg.clone() {
                    Long(arg) => format!("--{}", arg),
                    Short(arg) => format!("-{}", arg),
                    Value(arg) => arg.to_string_lossy().to_string(),
                };
                match arg {
                    Long("output") => {
                        next_output = true;
                        break;
                    }
                    Long("mode") | Long("resolution") => {
                        if output_builder.resolution.is_some() {
                            return Err(anyhow!("{arg_str} duplicated for output {name}"));
                        }
                        output_builder.resolution(p.value()?.parse()?);
                    }
                    Long("auto") | Long("preferred") => {
                        if output_builder.auto.is_some() {
                            return Err(anyhow!("{arg_str} duplicated for output {name}"));
                        }
                        output_builder.auto(true);
                    }
                    Long("off") => {
                        if output_builder.off.is_some() {
                            return Err(anyhow!("{arg_str} duplicated for output {name}"));
                        }
                        output_builder.off(true);
                    }
                    Short('r') | Long("rate") | Long("fps") => {
                        if output_builder.framerate.is_some() {
                            return Err(anyhow!("{arg_str} duplicated for output {name}"));
                        }
                        output_builder.framerate(p.value()?.parse()?);
                    }
                    _ => return Err(arg.unexpected().into()),
                }
            }

            let mode_group: Vec<_> = [
                output_builder.resolution.clone().map(|_| "resolution"),
                output_builder.auto.map(|_| "auto"),
                output_builder.off.map(|_| "off"),
            ]
            .into_iter()
            .flatten()
            .collect();
            if mode_group.len() > 1 {
                let mode_options = [
                    mode_group[..mode_group.len() - 1].join(", "),
                    mode_group.last().unwrap().to_string(),
                ]
                .join(" and ");
                return Err(anyhow!(
                    "using {mode_options} at the same time for output {name}"
                ));
            }

            outputs.push(output_builder.build()?);

            if next_output {
                continue;
            } else {
                break;
            }
        }
        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_by_output() {
        let args = Cli::parse(lexopt::Parser::from_iter(&[
            "gnome-randr",
            "--output",
            "HDMI-1",
            "--auto",
            "--output",
            "HDMI-2",
            "--off",
            "--output",
            "HDMI-3",
            "--mode",
            "1920x1080",
        ]))
        .unwrap();
        assert_eq!(
            args.outputs,
            &[
                OutputArgsBuilder::default()
                    .name("HDMI-1")
                    .auto(true)
                    .build()
                    .unwrap(),
                OutputArgsBuilder::default()
                    .name("HDMI-2")
                    .off(true)
                    .build()
                    .unwrap(),
                OutputArgsBuilder::default()
                    .name("HDMI-3")
                    .resolution(mode_db::Resolution {
                        width: 1920,
                        height: 1080
                    })
                    .build()
                    .unwrap(),
            ]
        )
    }

    #[test]
    fn no_duplicate_output() {
        let args = Cli::parse(lexopt::Parser::from_iter(&[
            "gnome-randr",
            "--output",
            "HDMI-1",
            "--output",
            "HDMI-1",
        ]));
        assert!(args.is_err());
        assert!(args.is_err_and(|err| { err.to_string().contains("HDMI-1") }));
    }

    #[test]
    fn no_mode_redefinition() {
        let args = Cli::parse(lexopt::Parser::from_iter(&[
            "gnome-randr",
            "--output",
            "HDMI-1",
            "--resolution",
            "1920x1080",
            "--auto",
        ]));
        assert!(args.is_err());
        assert!(args.is_err_and(|err| {
            let err = err.to_string();
            err.contains("resolution") && err.contains("auto")
        }));
    }
}
