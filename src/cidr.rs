use std::{
    fmt::{Debug, Display},
    net::Ipv4Addr,
    str::FromStr,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Cidr {
    network: Ipv4Addr,
    prefix: u8,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Error {
    CidrBoundsError(String),
    InvalidNetworkError(String),
    PrefixRangeError(String),
    ParseError(String),
    TypeCastError(String),
    ImpossibleError(String),
}

impl Cidr {
    pub fn new(network: Ipv4Addr, prefix: u8) -> Result<Self, Error> {
        if prefix as u32 > u32::BITS {
            return Err(Error::PrefixRangeError(format!(
                "network prefix '{prefix}' must be 32 or less"
            )));
        }
        if network
            .octets()
            .iter()
            .skip((prefix / 8).into())
            .enumerate()
            .any(|(i, &o)| {
                let offset = prefix % 8;
                (if i > 0 { o } else { o << offset >> offset }) != 0
            })
        {
            return Err(Error::InvalidNetworkError(format!(
                "network address '{network}' must be clear after the first {prefix} bits"
            )));
        }
        Ok(Self { network, prefix })
    }

    pub fn network(&self) -> Ipv4Addr {
        self.network
    }

    pub fn prefix(&self) -> u8 {
        self.prefix
    }

    pub fn first(&self) -> Ipv4Addr {
        self.network
    }

    pub fn mid(&self) -> Ipv4Addr {
        if self.prefix as u32 == u32::BITS {
            self.network
        } else {
            (u32::from(self.network) | (1 << (u32::BITS - self.prefix as u32 - 1))).into()
        }
    }

    pub fn last(&self) -> Ipv4Addr {
        let mut last = self.network.octets();
        let first_octet: usize = (self.prefix() / 8).into();
        for i in first_octet..last.len() {
            if i > first_octet {
                last[i] = u8::MAX
            } else {
                let offset = self.prefix % 8;
                last[i] |= u8::MAX << offset >> offset;
            }
        }
        Ipv4Addr::from(last)
    }

    pub fn contains<T>(&self, net: T) -> Result<bool, Error>
    where
        T: Copy + Debug + TryInto<Cidr>,
    {
        let cidr: Cidr = net.try_into().map_err(|_| {
            Error::TypeCastError(format!("could not cast value '{:?}' to cidr", net))
        })?;
        Ok(cidr.first() >= self.first() && cidr.last() <= self.last())
    }

    pub fn split(&self) -> Result<[Cidr; 2], Error> {
        let prefix = self.prefix + 1;
        Ok([
            Self::new(self.network, prefix)?,
            Self::new(self.mid(), prefix)?,
        ])
    }
}

impl Default for Cidr {
    fn default() -> Self {
        Self {
            network: Ipv4Addr::from(<[u8; 4]>::default()),
            prefix: Default::default(),
        }
    }
}

impl Display for Cidr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}/{}", self.network, self.prefix)
    }
}

impl TryFrom<Ipv4Addr> for Cidr {
    type Error = Error;

    fn try_from(value: Ipv4Addr) -> Result<Self, Self::Error> {
        Self::new(value, u32::BITS as u8)
    }
}

impl FromStr for Cidr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((network, prefix)) = s.split_once('/') {
            Self::new(
                network
                    .parse::<Ipv4Addr>()
                    .map_err(|e| Error::ParseError(e.to_string()))?,
                prefix
                    .parse::<u8>()
                    .map_err(|e| Error::ParseError(e.to_string()))?,
            )
        } else {
            Err(Error::ParseError(
                "missing network prefix delimiter".to_owned(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn cidr_constructor() {
    //     for prefix in 0..=32 {
    //         println!("{}", Cidr::new(Ipv4Addr::new(0b10000000, 0, 0, 0), prefix).unwrap());
    //         println!("{}", Cidr::new(Ipv4Addr::new(0xFF, 0xFF, 0xFF, 0xFF), prefix).unwrap());
    //     }
    // }

    // #[test]
    // fn cidr_first() {
    //     let cidr: Cidr = "10.0.0.0/8".parse().unwrap();
    //     println!("{} / {} : {} -> {}", cidr.network(), cidr.prefix(), cidr.first(), cidr.last());
    //     let cidr: Cidr = "10.0.0.0/9".parse().unwrap();
    //     println!("{} / {} : {} -> {}", cidr.network(), cidr.prefix(), cidr.first(), cidr.last());
    //     let cidr: Cidr = "10.128.0.0/9".parse().unwrap();
    //     println!("{} / {} : {} -> {}", cidr.network(), cidr.prefix(), cidr.first(), cidr.last());
    //     let cidr: Cidr = "10.128.0.0/8".parse().unwrap();
    //     println!("{} / {} : {} -> {}", cidr.network(), cidr.prefix(), cidr.first(), cidr.last());
    // }

    #[test]
    fn it_works() {
        // let c: Cidr = "10.0.0.0/8".parse().unwrap();
        // let [l, r] = c.split().unwrap();
        // println!("{l}, {r}");
        // for i in 0..=32 {
        //     println!("{} {}", i / 8, i % 8);
        // }
        // let o = 127_u8;
        // println!("{}", o == o >> 1 << 1);
        // println!("{}", "127.0.343.0".parse::<Ipv4Addr>().unwrap());
        // println!("{}", "127.0.343.0".parse::<Cidr>().unwrap());
    }
}
