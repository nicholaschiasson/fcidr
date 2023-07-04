# fcidr

[![crates.io](https://img.shields.io/crates/v/fcidr)](https://crates.io/crates/fcidr)

Fragmented Classless Inter-Domain Routing (FCIDR)

A library exposing a data structure to represent a set of CIDR ranges and
easily manipulate its entries using set-like operations.

This data structure can be applied, for example, in configuring firewalls that
*implicitly deny* (AWS Security Groups) using a rule set that explicitly
expresses rules for both allow and deny.
