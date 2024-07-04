use crate::cfg;
use anyhow::{anyhow, Result};
use std::io::{Read, Write};

#[derive(Debug)]
pub struct Header {
    magic: [u8; 11],
    version: u32,
    padding: u8,
}

impl Header {
    pub fn read(mut reader: impl Read) -> Result<Self> {
        let mut magic = [0; 11];
        let mut version = [0; 4];
        let mut pad = [0; 1];

        reader.read_exact(&mut magic)?;
        reader.read_exact(&mut version)?;
        reader.read_exact(&mut pad)?;

        Ok(Self {
            magic,
            version: u32::from_be_bytes(version),
            padding: pad[0],
        })
    }

    pub fn write(self, mut writer: impl Write) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&u32::to_be_bytes(self.version))?;
        writer.write_all(&[self.padding])?;

        Ok(())
    }

    pub fn validated(self) -> Result<Self> {
        if self.magic != Self::default().magic {
            return Err(anyhow!("invalid magic value"));
        }

        if self.version > Self::default().version {
            return Err(anyhow!(
                "unsupported version: world is v{}, but we support only up to \
                 v{}",
                self.version,
                cfg::VERSION,
            ));
        }

        if self.padding != Self::default().padding {
            return Err(anyhow!("invalid padding"));
        }

        Ok(self)
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: *b"kartoffels:",
            version: cfg::VERSION,
            padding: 0,
        }
    }
}
