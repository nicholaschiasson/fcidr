use std::{cell::RefCell, rc::Rc};

use crate::Cidr;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
enum Inclusion {
    #[default]
    Excluded,
    Included,
    Subnets([Rc<RefCell<CidrNode>>; 2]),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum BinarySetOperator {
    Difference,
    Union,
}

impl Into<Inclusion> for BinarySetOperator {
    fn into(self) -> Inclusion {
        match self {
            BinarySetOperator::Difference => Inclusion::Excluded,
            BinarySetOperator::Union => Inclusion::Included,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct CidrNode {
    cidr: Cidr,
    inclusion: Inclusion,
}

impl CidrNode {
    fn new(cidr: Cidr) -> Self {
        Self {
            cidr,
            inclusion: Default::default(),
        }
    }

    fn binary_set_operation(&mut self, cidr: Cidr, operator: BinarySetOperator) -> &mut Self {
        if self.cidr == cidr {
            self.inclusion = operator.into();
        } else if self.cidr.contains(cidr) && self.inclusion != operator.into() {
            let subnets = match &self.inclusion {
                Inclusion::Subnets([left, right]) => [left.clone(), right.clone()],
                inclusion => {
                    let [left, right] = [
                        Rc::new(RefCell::new(CidrNode {
                            cidr: self.cidr.left_subnet().unwrap(),
                            inclusion: inclusion.to_owned(),
                        })),
                        Rc::new(RefCell::new(CidrNode {
                            cidr: self.cidr.right_subnet().unwrap(),
                            inclusion: inclusion.to_owned(),
                        })),
                    ];
                    self.inclusion = Inclusion::Subnets([left.clone(), right.clone()]);
                    [left, right]
                }
            };
            for subnet in &subnets {
                subnet.borrow_mut().binary_set_operation(cidr, operator);
            }
            if subnets
                .iter()
                .all(|subnet| subnet.borrow().inclusion == operator.into())
            {
                self.inclusion = operator.into();
            }
        }
        self
    }

    fn contains(&self, cidr: Cidr) -> bool {
        if cidr.prefix() < self.cidr.prefix() {
            return false;
        }
        match &self.inclusion {
            Inclusion::Excluded => false,
            Inclusion::Included => self.cidr.contains(cidr),
            Inclusion::Subnets([left, right]) => {
                if cidr.network() < self.cidr.mid() {
                    left.borrow().contains(cidr)
                } else {
                    right.borrow().contains(cidr)
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Fcidr {
    cidr: Rc<RefCell<CidrNode>>,
}

impl Fcidr {
    pub fn new(cidr: Cidr) -> Self {
        let mut fcidr = Self::default();
        let mut next = vec![Rc::new(RefCell::new(CidrNode {
            cidr,
            inclusion: Inclusion::Included,
        }))];
        while let Some(n) = next.pop() {
            if let (Some(parent), cidr) = (n.borrow().cidr.parent(), n.borrow().cidr) {
                next.push(Rc::new(RefCell::new(CidrNode {
                    cidr: parent,
                    inclusion: Inclusion::Subnets(
                        if (u32::from(cidr.network()) >> (u32::BITS - cidr.prefix() as u32)) & 1
                            == 0
                        {
                            [
                                n.clone(),
                                Rc::new(RefCell::new(CidrNode::new(
                                    parent.right_subnet().unwrap(),
                                ))),
                            ]
                        } else {
                            [
                                Rc::new(RefCell::new(CidrNode::new(parent.left_subnet().unwrap()))),
                                n.clone(),
                            ]
                        },
                    ),
                })));
            } else {
                fcidr.cidr = n.clone();
            }
        }
        fcidr
    }

    pub fn complement(&mut self) -> &mut Self {
        let mut next = vec![self.cidr.clone()];
        while let Some(node) = next.pop() {
            let mut node = node.borrow_mut();
            match &node.inclusion {
                Inclusion::Excluded => node.inclusion = Inclusion::Included,
                Inclusion::Included => node.inclusion = Inclusion::Excluded,
                Inclusion::Subnets(subnet) => {
                    for s in subnet {
                        next.push(s.clone());
                    }
                }
            }
        }
        self
    }

    pub fn difference(&mut self, cidr: Cidr) -> &mut Self {
        self.cidr
            .borrow_mut()
            .binary_set_operation(cidr, BinarySetOperator::Difference);
        self
    }

    pub fn is_superset(&self, cidr: Cidr) -> bool {
        self.cidr.borrow().contains(cidr)
    }

    pub fn union(&mut self, cidr: Cidr) -> &mut Self {
        self.cidr
            .borrow_mut()
            .binary_set_operation(cidr, BinarySetOperator::Union);
        self
    }

    pub fn iter(&self) -> FcidrIntoIterator {
        FcidrIntoIterator {
            next: vec![self.cidr.clone()],
        }
    }
}

impl From<Cidr> for Fcidr {
    fn from(value: Cidr) -> Self {
        Self::new(value)
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

#[derive(Debug, Default)]
pub struct FcidrIntoIterator {
    next: Vec<Rc<RefCell<CidrNode>>>,
}

impl Iterator for FcidrIntoIterator {
    type Item = Cidr;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.next.pop() {
            match &node.borrow().inclusion {
                Inclusion::Excluded => continue,
                Inclusion::Included => return Some(node.borrow().cidr),
                Inclusion::Subnets(subnets) => {
                    for subnet in subnets.iter().rev().map(|s| s.to_owned()) {
                        self.next.push(subnet);
                    }
                }
            }
        }
        None
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     // #[test]
//     // fn does_it_work() {
//     //     let mut fcidr = Fcidr::default();
//     //     fcidr.iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     fcidr.complement().complement().iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     // println!("{fcidr:#?}\n");
//     //     println!();
//     //     let mut fcidr = Fcidr::new("0.0.0.0/0".parse().unwrap());
//     //     fcidr.iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     fcidr.complement().iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     // println!("{fcidr:#?}\n");
//     //     println!();
//     //     let mut fcidr = Fcidr::new("48.0.0.0/4".parse().unwrap());
//     //     fcidr.iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     fcidr.complement().iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     // println!("{fcidr:#?}\n");
//     //     println!();
//     //     let mut fcidr = Fcidr::new("10.0.128.0/25".parse().unwrap());
//     //     fcidr.iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     fcidr.complement().iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     // println!("{fcidr:#?}\n");
//     //     println!();
//     //     let mut fcidr = Fcidr::new("255.255.255.255/32".parse().unwrap());
//     //     fcidr.iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     fcidr.complement().iter().for_each(|c| println!("{c}"));
//     //     println!();
//     //     // println!("{fcidr:#?}\n");
//     //     println!();
//     // }

//     #[test]
//     fn it_works() {
//         let mut fcidr = Fcidr::default();
//         fcidr.union("10.0.0.0/8".parse().unwrap());
//         fcidr.union("10.0.128.0/24".parse().unwrap());
//         fcidr.difference("10.0.80.0/20".parse().unwrap());
//         fcidr.union("10.0.82.0/24".parse().unwrap());
//         // fcidr.union("10.0.0.0/24".parse().unwrap());
//         // fcidr.union("10.0.128.0/25".parse().unwrap());
//         // fcidr.union("11.0.0.0/8".parse().unwrap());
//         // fcidr.difference("10.0.0.64/32".parse().unwrap());
//         // fcidr.union("10.0.0.64/32".parse().unwrap());
//         // fcidr.difference("10.0.0.64/32".parse().unwrap());
//         // fcidr.union("0.0.0.0/0".parse().unwrap());
//         // fcidr.difference("128.0.0.0/32".parse().unwrap());
//         // fcidr
//         //     .difference("255.255.255.255/32".parse().unwrap());
//         // fcidr.union("0.0.0.0/0".parse().unwrap());
//         // fcidr.difference("10.0.0.1/32".parse().unwrap());
//         // println!("{:?}", fcidr.iter().collect::<Vec<_>>());
//         for cidr in &fcidr {
//             println!("{cidr}");
//         }
//         println!("{fcidr:?}");
//     }
// }
