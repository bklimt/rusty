use std::collections::HashMap;

use crate::proto_type::{infer_proto_type, ProtoType};

use anyhow::{anyhow, Result};
use proc_macro2::Ident;
use syn::{DataStruct, LitInt};

#[derive(Debug)]
pub struct FieldDesc {
    pub id: u64,
    pub name: Ident,
    pub ty: ProtoType,
}

pub fn extract_fields(data: DataStruct) -> Result<Vec<FieldDesc>> {
    let mut fields = Vec::new();

    for field in data.fields.iter() {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| anyhow!("no ident for field"))?
            .clone();

        let type_attr = field.attrs.iter().find(|attr| attr.path.is_ident("pbtype"));
        let type_attr = if let Some(attr) = type_attr {
            let id: Ident = attr.parse_args()?;
            let s = id.to_string();
            let pt = ProtoType::from_str(&s).ok_or_else(|| anyhow!("invalid proto type {}", s))?;
            Some(pt)
        } else {
            None
        };

        let type_inferred = infer_proto_type(&field.ty)?;

        let ty = type_attr.unwrap_or(type_inferred);

        let id_attr = field
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("id"))
            .ok_or_else(|| anyhow!("no id attribute for field {}", ident.to_string()))?;

        let sid: LitInt = id_attr.parse_args()?;
        let id: u64 = sid.base10_parse()?;
        fields.push(FieldDesc {
            id,
            name: ident.clone(),
            ty: ty.clone(),
        });
    }

    Ok(fields)
}
