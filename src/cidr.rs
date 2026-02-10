use std::{
    fmt::{Debug, Display},
    net::Ipv4Addr,
    str::FromStr,
};

use crate::Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Cidr {
    network: Ipv4Addr,
    prefix: u8,
}

impl Cidr {
    pub fn new(network: Ipv4Addr, prefix: u8) -> Result<Self, Error> {
        if prefix as u32 > u32::BITS {
            return Err(Error::InvalidPrefix(format!(
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
            return Err(Error::InvalidNetwork(format!(
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
        for (i, o) in last.iter_mut().skip(first_octet).enumerate() {
            if i > 0 {
                *o = u8::MAX
            } else {
                let offset = self.prefix % 8;
                *o |= u8::MAX << offset >> offset;
            }
        }
        Ipv4Addr::from(last)
    }

    pub fn contains<T>(&self, net: T) -> bool
    where
        T: Copy + Debug + Into<Cidr>,
    {
        let cidr: Cidr = net.into();
        cidr.first() >= self.first() && cidr.last() <= self.last()
    }

    pub fn parent(&self) -> Option<Cidr> {
        match self.prefix {
            0 => None,
            1 => Some(Self::default()),
            _ => {
                let prefix = self.prefix - 1;
                let shift = u32::BITS - prefix as u32;
                Some(Self {
                    network: (u32::from(self.network) >> shift << shift).into(),
                    prefix,
                })
            }
        }
    }

    pub fn left_subnet(&self) -> Option<Cidr> {
        match self.prefix as u32 {
            u32::BITS => None,
            _ => Some(Self {
                network: self.network,
                prefix: self.prefix + 1,
            }),
        }
    }

    pub fn right_subnet(&self) -> Option<Cidr> {
        match self.prefix as u32 {
            u32::BITS => None,
            _ => {
                let prefix = self.prefix + 1;
                let shift = u32::BITS - prefix as u32;
                Some(Self {
                    network: (((u32::from(self.network) >> shift) | 1) << shift).into(),
                    prefix,
                })
            }
        }
    }

    pub fn split(&self) -> Option<[Cidr; 2]> {
        match (self.left_subnet(), self.right_subnet()) {
            (Some(left), Some(right)) => Some([left, right]),
            _ => None,
        }
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

impl From<Ipv4Addr> for Cidr {
    fn from(value: Ipv4Addr) -> Self {
        Self::new(value, u32::BITS as u8).expect("convert from Ipv4Addr")
    }
}

// impl TryFrom<Ipv4Addr> for Cidr {
//     type Error = Error;

//     fn try_from(value: Ipv4Addr) -> Result<Self, Self::Error> {
//         Self::new(value, u32::BITS as u8)
//     }
// }

impl FromStr for Cidr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((network, prefix)) = s.split_once('/') {
            Self::new(
                network
                    .parse::<Ipv4Addr>()
                    .map_err(|e| Error::Parse(e.to_string()))?,
                prefix
                    .parse::<u8>()
                    .map_err(|e| Error::Parse(e.to_string()))?,
            )
        } else {
            Err(Error::Parse("missing network prefix delimiter".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_prefix_rejected() {
        let err = Cidr::new(Ipv4Addr::new(0, 0, 0, 0), 33).unwrap_err();
        matches!(err, Error::InvalidPrefix(_));
    }

    #[test]
    fn invalid_network_bits_rejected() {
        // host bits present for /8 must be zero
        let err = Cidr::new(Ipv4Addr::new(10, 0, 0, 1), 8).unwrap_err();
        matches!(err, Error::InvalidNetwork(_));
    }

    #[test]
    fn boundaries_and_midpoint() {
        let cidr: Cidr = "10.0.0.0/8".parse().unwrap();
        assert_eq!(cidr.first(), Ipv4Addr::new(10, 0, 0, 0));
        assert_eq!(cidr.mid(), Ipv4Addr::new(10, 128, 0, 0));
        assert_eq!(cidr.last(), Ipv4Addr::new(10, 255, 255, 255));
    }

    #[test]
    fn parent_and_subnets() {
        let cidr: Cidr = "10.0.0.0/8".parse().unwrap();
        let left = cidr.left_subnet().unwrap();
        let right = cidr.right_subnet().unwrap();
        assert_eq!(left.to_string(), "10.0.0.0/9");
        assert_eq!(right.to_string(), "10.128.0.0/9");

        let parent = left.parent().unwrap();
        assert_eq!(parent.to_string(), "10.0.0.0/8");
        assert!(Cidr::default().parent().is_none()); // /0 has no parent
    }

    #[test]
    fn contains_checks() {
        let cidr: Cidr = "10.0.0.0/8".parse().unwrap();
        assert!(cidr.contains("10.80.0.0/16".parse::<Cidr>().unwrap()));
        assert!(cidr.contains(Ipv4Addr::new(10, 0, 0, 1))); // /32 derived from IP
        assert!(!cidr.contains("11.0.0.0/8".parse::<Cidr>().unwrap()));
    }
}
