use color_eyre::{eyre::eyre, Result};
use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum MediaType {
    Movie,
    Tv,
}

impl Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Movie => write!(f, "Movie"),
            Self::Tv => write!(f, "TV"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Order {
    Desc,
    Asc,
}

#[derive(Debug, Clone, Copy)]
pub enum SortingValue {
    Name,
    Size,
    Type,
}

#[derive(Debug, Clone)]
pub struct SortingOption {
    pub sorting_value: SortingValue,
    pub sorting_direction: Order,
}

impl Default for SortingOption {
    fn default() -> Self {
        SortingOption {
            sorting_value: SortingValue::Name,
            sorting_direction: Order::Asc,
        }
    }
}

impl SortingOption {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "nd" => {
                Ok(SortingOption {
                    sorting_value: SortingValue::Name,
                    sorting_direction: Order::Desc,
                })
            }
            "n" => {
                Ok(SortingOption {
                    sorting_value: SortingValue::Name,
                    sorting_direction: Order::Asc,
                })
            }
            "sa" => {
                Ok(SortingOption {
                    sorting_value: SortingValue::Size,
                    sorting_direction: Order::Asc,
                })
            }
            "s" => {
                Ok(SortingOption {
                    sorting_value: SortingValue::Size,
                    sorting_direction: Order::Desc,
                })
            }
            "t" => {
                Ok(SortingOption {
                    sorting_value: SortingValue::Type,
                    sorting_direction: Order::Desc,
                })
            }
            _ => Err(eyre!("Not a valid Sorting Option")),
        }
    }
}
