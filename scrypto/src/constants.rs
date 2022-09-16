use crate::component::{ComponentAddress, PackageAddress};
use crate::resource::*;
use crate::{address, construct_address};

/// The address of the sys-faucet package.
pub const SYS_FAUCET_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    72,
    223,
    194,
    44,
    177,
    98,
    231,
    38,
    12,
    132,
    2,
    197,
    57,
    40,
    72,
    34,
    129,
    17,
    124,
    16,
    161,
    221,
    137,
    22,
    103,
    240
);
/// The address of the sys-utils package.
pub const SYS_UTILS_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    0,
    44,
    100,
    204,
    153,
    17,
    167,
    139,
    223,
    159,
    221,
    222,
    95,
    90,
    157,
    196,
    136,
    236,
    235,
    197,
    213,
    35,
    187,
    15,
    207,
    158
);

/// The address of the account package.
pub const ACCOUNT_PACKAGE: PackageAddress = construct_address!(
    EntityType::Package,
    117,
    149,
    161,
    192,
    155,
    192,
    68,
    56,
    79,
    186,
    128,
    155,
    199,
    188,
    92,
    59,
    83,
    241,
    146,
    178,
    126,
    213,
    55,
    167,
    164,
    201
);

/// The address of the SysFaucet component
pub const SYS_FAUCET_COMPONENT: ComponentAddress = construct_address!(
    EntityType::NormalComponent,
    173, 130, 50, 141, 112, 34, 61, 91, 174, 38, 130, 96, 179, 4, 93, 204, 113, 220, 243, 95, 55, 167, 67, 74, 9, 105
);

pub const SYS_SYSTEM_COMPONENT: ComponentAddress = construct_address!(
    EntityType::SystemComponent,
    141, 129, 247, 20, 46, 8, 166, 23, 225, 192, 118, 147, 168, 25, 252, 113, 41, 42, 140, 141, 169, 183, 148, 102, 224, 208
);
// TODO Add other system components

/// The system token which allows access to system resources (e.g. setting epoch)
pub const SYSTEM_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    185,
    23,
    55,
    238,
    138,
    77,
    229,
    157,
    73,
    218,
    212,
    13,
    229,
    86,
    14,
    87,
    84,
    70,
    106,
    200,
    76,
    245,
    67,
    46,
    169,
    93
);

/// The ECDSA virtual resource address.
pub const ECDSA_TOKEN: ResourceAddress = construct_address!(
    EntityType::Resource,
    43,
    113,
    132,
    253,
    47,
    66,
    111,
    180,
    52,
    199,
    68,
    195,
    33,
    205,
    145,
    223,
    131,
    117,
    181,
    225,
    240,
    27,
    116,
    0,
    157,
    255
);

/// The ED25519 virtual resource address.
pub const ED25519_TOKEN: ResourceAddress = address!(EntityType::Resource, 3u8);

/// The XRD resource address.
pub const RADIX_TOKEN: ResourceAddress = address!(
    EntityType::Resource,
    143,
    46,
    234,
    87,
    25,
    53,
    120,
    228,
    5,
    237,
    56,
    58,
    19,
    153,
    205,
    168,
    37,
    196,
    182,
    161,
    162,
    189,
    144,
    106,
    252,
    99
);
