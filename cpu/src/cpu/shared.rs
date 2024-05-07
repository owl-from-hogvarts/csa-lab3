use std::fmt::Display;

#[derive(Debug)]
pub struct Status {
    pub zero: bool,
    pub carry: bool,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.zero {
            write!(f, "ZERO")?;
        }

        if self.carry {
            write!(f, ", CARRY")?;
        }

        Ok(())
    }
}
