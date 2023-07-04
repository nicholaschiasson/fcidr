mod cidr;

use std::{cell::RefCell, fmt::Display, rc::Rc};

pub use cidr::Cidr;

#[derive(Clone, Debug, Default)]
pub enum FcidrInclusion {
    #[default]
    Excluded,
    Included,
    Subnets([Option<Rc<RefCell<Fcidr>>>; 2]),
}

#[derive(Clone, Debug, Default)]
pub struct Fcidr {
    cidr: Cidr,
    inclusion: FcidrInclusion,
}

impl Fcidr {
    pub fn new(cidr: Cidr) -> Result<Self, cidr::Error> {
        let mut fcidr = Self::default();
        fcidr.include(cidr)?;
        Ok(fcidr)
    }

    fn new_subnet(child: Cidr) -> Self {
        Self {
            cidr: child,
            ..Default::default()
        }
    }

    pub fn exclude(&mut self, cidr: Cidr) -> Result<(), cidr::Error> {
        if self.cidr == cidr {
            self.inclusion = FcidrInclusion::Excluded;
            return Ok(());
        }

        if !self.cidr.contains(cidr)? {
            return Err(cidr::Error::CidrBoundsError(format!(
                "cidr '{}' cannot exclude '{}' which it does not contain",
                self.cidr, cidr
            )));
        }

        if let FcidrInclusion::Included = self.inclusion {
            for cidr in self.cidr.split()? {
                self.include(cidr)?;
            }
        }

        if let FcidrInclusion::Subnets(_) = self.inclusion {
        } else {
            self.inclusion = FcidrInclusion::Subnets([None, None]);
        }

        let prefix = self.cidr.prefix() + 1;

        if prefix as u32 > u32::BITS {
            return Err(cidr::Error::PrefixRangeError(format!(
                "network prefix '{}' must be 32 or less",
                cidr.prefix()
            )));
        }

        let index = ((u32::from(cidr.network()) >> (u32::BITS - prefix as u32)) & 1) as usize;

        let subnet = match (index & 1, &mut self.inclusion) {
            (0, FcidrInclusion::Subnets([Some(subnet), _]))
            | (1, FcidrInclusion::Subnets([_, Some(subnet)])) => subnet.clone(),
            (_, FcidrInclusion::Subnets(subnets)) => {
                let subnet = Rc::new(RefCell::new(Fcidr::new_subnet(self.cidr.split()?[index])));
                subnets[index] = Some(subnet.clone());
                subnet
            }
            (_, inclusion) => {
                return Err(cidr::Error::ImpossibleError(format!(
                    "inclusion state is '{inclusion:?}'"
                )))
            }
        };

        let res = (*subnet).borrow_mut().exclude(cidr);
        res
    }

    pub fn include(&mut self, cidr: Cidr) -> Result<(), cidr::Error> {
        if self.cidr == cidr {
            self.inclusion = FcidrInclusion::Included;
            return Ok(());
        }

        if !self.cidr.contains(cidr)? {
            return Err(cidr::Error::CidrBoundsError(format!(
                "cidr '{}' cannot include '{}' which it does not contain",
                self.cidr, cidr
            )));
        }

        if let FcidrInclusion::Excluded = self.inclusion {
            for cidr in self.cidr.split()? {
                self.exclude(cidr)?;
            }
        }

        if let FcidrInclusion::Subnets(_) = self.inclusion {
        } else {
            self.inclusion = FcidrInclusion::Subnets([None, None]);
        }

        let prefix = self.cidr.prefix() + 1;

        if prefix as u32 > u32::BITS {
            return Err(cidr::Error::PrefixRangeError(format!(
                "network prefix '{}' must be 32 or less",
                cidr.prefix()
            )));
        }

        let index = ((u32::from(cidr.network()) >> (u32::BITS - prefix as u32)) & 1) as usize;

        let subnet = match (index & 1, &mut self.inclusion) {
            (0, FcidrInclusion::Subnets([Some(subnet), _]))
            | (1, FcidrInclusion::Subnets([_, Some(subnet)])) => subnet.clone(),
            (_, FcidrInclusion::Subnets(subnets)) => {
                let subnet = Rc::new(RefCell::new(Fcidr::new_subnet(self.cidr.split()?[index])));
                subnets[index] = Some(subnet.clone());
                subnet
            }
            (_, inclusion) => {
                return Err(cidr::Error::ImpossibleError(format!(
                    "inclusion state is '{inclusion:?}'"
                )))
            }
        };

        let res = (*subnet).borrow_mut().include(cidr);
        res
    }

    pub fn iter(&self) -> FcidrIntoIterator {
        match &self.inclusion {
            FcidrInclusion::Excluded => FcidrIntoIterator {
                next: None,
                remaining: Vec::new(),
            },
            FcidrInclusion::Included => FcidrIntoIterator {
                next: Some(self.cidr),
                remaining: Vec::new(),
            },
            FcidrInclusion::Subnets(subnets) => FcidrIntoIterator {
                next: None,
                remaining: subnets
                    .iter()
                    .rev()
                    .flatten()
                    .map(|s| s.to_owned())
                    .collect(),
            },
        }
    }
}

impl Display for Fcidr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for cidr in self.into_iter() {
            writeln!(f, "{cidr}")?;
        }
        Ok(())
    }
}

impl IntoIterator for Fcidr {
    type Item = Cidr;

    type IntoIter = FcidrIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for &Fcidr {
    type Item = Cidr;

    type IntoIter = FcidrIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl TryFrom<Cidr> for Fcidr {
    type Error = cidr::Error;

    fn try_from(value: Cidr) -> Result<Self, Self::Error> {
        todo!()
    }
}

pub struct FcidrIntoIterator {
    // TODO: Get rid of this when datastructure uses actual tree node for root
    next: Option<Cidr>,
    remaining: Vec<Rc<RefCell<Fcidr>>>,
}

impl Iterator for FcidrIntoIterator {
    type Item = Cidr;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Get rid of this when datastructure uses actual tree node for root
        if let Some(next) = self.next {
            self.next = None;
            return Some(next);
        }

        while let Some(fcidr) = self.remaining.pop() {
            let fcidr = (*fcidr).borrow();
            match &fcidr.inclusion {
                FcidrInclusion::Excluded => continue,
                FcidrInclusion::Included => return Some(fcidr.cidr),
                FcidrInclusion::Subnets(subnets) => {
                    for subnet in subnets.iter().rev().flatten().map(|s| s.to_owned()) {
                        self.remaining.push(subnet);
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut fcidr = Fcidr::default();
        fcidr.include("10.0.0.0/24".parse().unwrap()).unwrap();
        fcidr.include("10.0.128.0/25".parse().unwrap()).unwrap();
        fcidr.include("11.0.0.0/8".parse().unwrap()).unwrap();
        fcidr.exclude("10.0.0.64/32".parse().unwrap()).unwrap();
        fcidr.include("0.0.0.0/0".parse().unwrap()).unwrap();
        fcidr.exclude("128.0.0.0/32".parse().unwrap()).unwrap();
        fcidr
            .exclude("255.255.255.255/32".parse().unwrap())
            .unwrap();
        // fcidr.include("0.0.0.0/0".parse().unwrap()).unwrap();
        println!("{fcidr}");
        // println!("{:?}", fcidr.iter().collect::<Vec<_>>());
        println!("{fcidr:?}");
        // fcidr.exclude("10.0.0.1/32".parse().unwrap());
        // for i in 0..=32 {
        //     println!("{} {}", i / 8, i % 8);
        // }
        // let o = 127_u8;
        // println!("{}", o == o >> 1 << 1);
        // println!("{}", "127.0.343.0".parse::<Ipv4Addr>().unwrap());
        // println!("{}", "127.0.343.0".parse::<Cidr>().unwrap());
    }
}
