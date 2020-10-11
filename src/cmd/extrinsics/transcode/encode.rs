// Copyright 2018-2020 Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

use anyhow::Result;
use ron::{Number, Value};
use scale::{Encode, Output};
use scale_info::{
    form::CompactForm, Field, RegistryReadOnly, Type, TypeDef, TypeDefArray, TypeDefComposite,
    TypeDefPrimitive,
};
use std::{convert::TryInto, fmt::Debug, str::FromStr};

use super::resolve_type;

pub trait EncodeValue {
    fn encode_value_to<O: Output + Debug>(
        &self,
        registry: &RegistryReadOnly,
        value: &ron::Value,
        output: &mut O,
    ) -> Result<()>;
}

impl EncodeValue for Type<CompactForm> {
    fn encode_value_to<O: Output + Debug>(
        &self,
        registry: &RegistryReadOnly,
        value: &Value,
        output: &mut O,
    ) -> Result<()> {
        self.type_def().encode_value_to(registry, value, output)
    }
}

impl EncodeValue for TypeDef<CompactForm> {
    fn encode_value_to<O: Output + Debug>(
        &self,
        registry: &RegistryReadOnly,
        value: &Value,
        output: &mut O,
    ) -> Result<()> {
        match self {
            TypeDef::Array(array) => array.encode_value_to(registry, value, output),
            TypeDef::Primitive(primitive) => primitive.encode_value_to(registry, value, output),
            TypeDef::Composite(composite) => composite.encode_value_to(registry, value, output),
            def => unimplemented!("TypeDef::encode_value {:?}", def),
        }
    }
}

impl EncodeValue for TypeDefArray<CompactForm> {
    fn encode_value_to<O: Output + Debug>(
        &self,
        registry: &RegistryReadOnly,
        value: &Value,
        output: &mut O,
    ) -> Result<()> {
        let ty = resolve_type(registry, self.type_param())?;
        match value {
            Value::String(s) => {
                if *ty.type_def() == TypeDef::Primitive(TypeDefPrimitive::U8) {
                    let decoded_byte_string = hex::decode(s.trim_start_matches("0x"))?;
                    for byte in decoded_byte_string {
                        byte.encode_to(output);
                    }
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Only byte (u8) arrays supported as strings"
                    ))
                }
            }
            Value::Seq(values) => {
                for value in values {
                    ty.encode_value_to(registry, value, output)?;
                }
                Ok(())
            }
            value => Err(anyhow::anyhow!("{:?} cannot be encoded as an array", value)),
        }
    }
}

impl EncodeValue for TypeDefPrimitive {
    fn encode_value_to<O: Output + Debug>(
        &self,
        _: &RegistryReadOnly,
        value: &Value,
        output: &mut O,
    ) -> Result<()> {
        match self {
            TypeDefPrimitive::Bool => {
                if let ron::Value::Bool(b) = value {
                    b.encode_to(output);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Expected a bool value"))
                }
            }
            TypeDefPrimitive::Char => Err(anyhow::anyhow!("scale codec not implemented for char")),
            TypeDefPrimitive::Str => {
                if let ron::Value::String(s) = value {
                    s.encode_to(output);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Expected a String value"))
                }
            }
            TypeDefPrimitive::U8 => {
                if let ron::Value::Number(ron::Number::Integer(i)) = value {
                    let u: u8 = (*i).try_into()?;
                    u.encode_to(output);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Expected a u8 value"))
                }
            }
            TypeDefPrimitive::U16 => {
                if let ron::Value::Number(ron::Number::Integer(i)) = value {
                    let u: u16 = (*i).try_into()?;
                    u.encode_to(output);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Expected a u16 value"))
                }
            }
            TypeDefPrimitive::U32 => {
                if let ron::Value::Number(ron::Number::Integer(i)) = value {
                    let u: u32 = (*i).try_into()?;
                    u.encode_to(output);
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Expected a u16 value"))
                }
            }
            TypeDefPrimitive::U64 => match value {
                Value::Number(Number::Integer(i)) => {
                    let u: u64 = (*i).try_into()?;
                    u.encode_to(output);
                    Ok(())
                }
                Value::String(s) => {
                    let sanitized = s.replace(&['_', ','][..], "");
                    let u: u64 = u64::from_str(&sanitized)?;
                    u.encode_to(output);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Expected a Number or a String value")),
            },
            TypeDefPrimitive::U128 => match value {
                Value::Number(Number::Integer(i)) => {
                    let u: u128 = (*i).try_into()?;
                    u.encode_to(output);
                    Ok(())
                }
                Value::String(s) => {
                    let sanitized = s.replace(&['_', ','][..], "");
                    let u: u128 = u128::from_str(&sanitized)?;
                    u.encode_to(output);
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Expected a Number or a String value")),
            },

            _ => unimplemented!("TypeDefPrimitive::encode_value"),
            // TypeDefPrimitive::I8 => Ok(i8::encode(&i8::from_str(arg)?)),
            // TypeDefPrimitive::I16 => Ok(i16::encode(&i16::from_str(arg)?)),
            // TypeDefPrimitive::I32 => Ok(i32::encode(&i32::from_str(arg)?)),
            // TypeDefPrimitive::I64 => Ok(i64::encode(&i64::from_str(arg)?)),
            // TypeDefPrimitive::I128 => Ok(i128::encode(&i128::from_str(arg)?)),
        }
    }
}

impl EncodeValue for TypeDefComposite<CompactForm> {
    fn encode_value_to<O: Output + Debug>(
        &self,
        registry: &RegistryReadOnly,
        value: &Value,
        output: &mut O,
    ) -> Result<()> {
        if let Value::Map(map) = value {
            for (field, value) in self.fields().iter().zip(map.values()) {
                field.encode_value_to(registry, value, output)?;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Expected a Value::Map for a struct, found {:?}",
                value
            ))
        }
    }
}

impl EncodeValue for Field<CompactForm> {
    fn encode_value_to<O: Output + Debug>(
        &self,
        registry: &RegistryReadOnly,
        value: &Value,
        output: &mut O,
    ) -> Result<()> {
        let ty = resolve_type(registry, self.ty())?;
        ty.encode_value_to(registry, value, output)
    }
}
