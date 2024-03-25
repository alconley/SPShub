use super::channel_map::{ChannelMap, ChannelType};
#[allow(unused_imports)]
use super::compass_data::{decompose_uuid_to_board_channel, CompassData};
use super::used_size::UsedSize;
use std::collections::BTreeMap;
use std::hash::Hash;

use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumCount, EnumIter};

use polars::prelude::*;

const INVALID_VALUE: f64 = -1.0e6;

#[derive(Debug, Clone, Hash, Eq, PartialOrd, Ord, PartialEq, EnumIter, EnumCount, AsRefStr)]
pub enum ChannelDataField {
    Cebra0Energy,
    Cebra1Energy,
    Cebra2Energy,
    Cebra3Energy,
    Cebra4Energy,
    Cebra5Energy,
    Cebra6Energy,

    Cebra0Short,
    Cebra1Short,
    Cebra2Short,
    Cebra3Short,
    Cebra4Short,
    Cebra5Short,
    Cebra6Short,

    Cebra0Time,
    Cebra1Time,
    Cebra2Time,
    Cebra3Time,
    Cebra4Time,
    Cebra5Time,
    Cebra6Time,
}

impl ChannelDataField {
    //Returns a list of fields for iterating over
    pub fn get_field_vec() -> Vec<ChannelDataField> {
        ChannelDataField::iter().collect()
    }
}

impl UsedSize for ChannelDataField {
    fn get_used_size(&self) -> usize {
        std::mem::size_of::<ChannelDataField>()
    }
}

#[derive(Debug, Clone)]
pub struct ChannelData {
    //Columns must always come in same order, so use sorted map
    pub fields: BTreeMap<ChannelDataField, Vec<f64>>,
    pub rows: usize,
}

impl Default for ChannelData {
    fn default() -> Self {
        let fields = ChannelDataField::get_field_vec();
        let mut data = ChannelData {
            fields: BTreeMap::new(),
            rows: 0,
        };
        fields.into_iter().for_each(|f| {
            data.fields.insert(f, vec![]);
        });
        data
    }
}

impl UsedSize for ChannelData {
    fn get_used_size(&self) -> usize {
        self.fields.get_used_size()
    }
}

impl ChannelData {
    //To keep columns all same length, push invalid values as necessary
    fn push_defaults(&mut self) {
        for field in self.fields.iter_mut() {
            if field.1.len() < self.rows {
                field.1.push(INVALID_VALUE)
            }
        }
    }

    //Update the last element to the given value
    fn set_value(&mut self, field: &ChannelDataField, value: f64) {
        if let Some(list) = self.fields.get_mut(field) {
            if let Some(back) = list.last_mut() {
                *back = value;
            }
        }
    }

    pub fn append_event(&mut self, event: Vec<CompassData>, map: &ChannelMap) {
        self.rows += 1;
        self.push_defaults();

        for hit in event.iter() {
            //Fill out detector fields using channel map
            let channel_data = match map.get_channel_data(&hit.uuid) {
                Some(data) => data,
                None => continue,
            };
            match channel_data.channel_type {
                ChannelType::Cebra0 => {
                    self.set_value(&ChannelDataField::Cebra0Energy, hit.energy);
                    self.set_value(&ChannelDataField::Cebra0Short, hit.energy_short);
                    self.set_value(&ChannelDataField::Cebra0Time, hit.timestamp);
                }

                ChannelType::Cebra1 => {
                    self.set_value(&ChannelDataField::Cebra1Energy, hit.energy);
                    self.set_value(&ChannelDataField::Cebra1Short, hit.energy_short);
                    self.set_value(&ChannelDataField::Cebra1Time, hit.timestamp);
                }

                ChannelType::Cebra2 => {
                    self.set_value(&ChannelDataField::Cebra2Energy, hit.energy);
                    self.set_value(&ChannelDataField::Cebra2Short, hit.energy_short);
                    self.set_value(&ChannelDataField::Cebra2Time, hit.timestamp);
                }

                ChannelType::Cebra3 => {
                    self.set_value(&ChannelDataField::Cebra3Energy, hit.energy);
                    self.set_value(&ChannelDataField::Cebra3Short, hit.energy_short);
                    self.set_value(&ChannelDataField::Cebra3Time, hit.timestamp);
                }

                ChannelType::Cebra4 => {
                    self.set_value(&ChannelDataField::Cebra4Energy, hit.energy);
                    self.set_value(&ChannelDataField::Cebra4Short, hit.energy_short);
                    self.set_value(&ChannelDataField::Cebra4Time, hit.timestamp);
                }

                ChannelType::Cebra5 => {
                    self.set_value(&ChannelDataField::Cebra5Energy, hit.energy);
                    self.set_value(&ChannelDataField::Cebra5Short, hit.energy_short);
                    self.set_value(&ChannelDataField::Cebra5Time, hit.timestamp);
                }

                ChannelType::Cebra6 => {
                    self.set_value(&ChannelDataField::Cebra6Energy, hit.energy);
                    self.set_value(&ChannelDataField::Cebra6Short, hit.energy_short);
                    self.set_value(&ChannelDataField::Cebra6Time, hit.timestamp);
                }

                _ => continue,
            }
        }
    }

    pub fn convert_to_series(self) -> Vec<Series> {
        let sps_cols: Vec<Series> = self
            .fields
            .into_iter()
            .map(|field| -> Series { Series::new(field.0.as_ref(), field.1) })
            .collect();

        sps_cols
    }
}
