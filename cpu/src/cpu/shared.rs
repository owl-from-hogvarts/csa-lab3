use std::fmt::Display;

#[derive(Debug)]
pub struct Status {
    pub zero: bool,
    pub carry: bool,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags: Vec<String> = Vec::new();

        if self.zero {
            flags.push("ZERO".to_owned());
        }

        if self.carry {
            flags.push("CARRY".to_owned());
        }

        write!(
            f,
            "{}",
            flags
                .into_iter()
                .reduce(|acc, value| acc + ", " + &value)
                .unwrap_or_default()
        )?;

        Ok(())
    }
}
